import { afterEach, describe, expect, it, vi } from 'vitest';
import { livePreview } from '$lib/api/preview';
import { neutralEdits, neutralGeometry, type Edits } from '$lib/types/edits';

vi.mock('$lib/api/preview', () => ({
  livePreview: vi.fn(() => new Promise<never>(() => undefined))
}));

import {
  commit,
  cropIsDefault,
  endSession,
  flipAspect,
  refreshFrame,
  reset,
  rotateStep,
  scheduleCropCommit,
  setAspect,
  startSession,
  syncSession,
  updateDraftAngle,
  updateDraftCrop,
  type GeometryCtx
} from './geometry.svelte';

type TestCtx = GeometryCtx & { actions: string[] };

let open: TestCtx | null = null;

afterEach(async () => {
  if (open) await endSession(open);
  open = null;
  vi.useRealTimers();
  vi.mocked(livePreview).mockClear();
});

function session(loaded = true): TestCtx {
  const c: TestCtx = {
    assetId: 'asset-1',
    initialised: true,
    edits: neutralEdits() as Edits,
    geometrySession: null,
    error: null,
    actions: [],
    clearView: () => undefined,
    onCommit: async (action?: string) => {
      c.actions.push(action ?? '');
    }
  };
  startSession(c);
  open = c;
  if (loaded && c.geometrySession) {
    c.geometrySession.srcW = 600;
    c.geometrySession.srcH = 400;
  }
  return c;
}

describe('live geometry', () => {
  it('commits a changed gesture once and skips an unchanged one', async () => {
    const c = session();
    updateDraftAngle(c, 5);
    await commit(c, 'Straighten');
    await commit(c, 'Straighten');
    expect(c.edits.geometry.rotate_angle).toBe(5);
    expect(c.actions).toEqual(['Straighten']);
  });

  it('commits rotate and aspect changes as their own steps', async () => {
    const c = session();
    rotateStep(c, 90);
    await setAspect(c, { kind: 'ratio', num: 1, den: 1 });
    expect(c.edits.geometry.rotate).toBe(90);
    expect(c.edits.geometry.aspect).toEqual({ kind: 'ratio', num: 1, den: 1 });
    expect(c.actions).toEqual(['Rotate', 'Aspect ratio']);
  });

  it('ignores draft gestures until the frame has loaded', async () => {
    const c = session(false);
    updateDraftAngle(c, 5);
    await commit(c, 'Straighten');
    expect(c.actions).toEqual([]);
  });

  it('resets the drafts when the edits change outside the tab', async () => {
    const c = session();
    updateDraftAngle(c, 5);
    await commit(c, 'Straighten');
    c.edits = neutralEdits() as Edits;
    syncSession(c, c.edits.geometry);
    expect(c.geometrySession?.draftAngle).toBe(0);
  });

  it('keeps a hand-made crop when the edits already match the drafts', async () => {
    const c = session();
    updateDraftCrop(c, { x: 0.1, y: 0.1, w: 0.5, h: 0.5 });
    await commit(c, 'Crop');
    syncSession(c, c.edits.geometry);
    expect(c.geometrySession?.userEditedCrop).toBe(true);
  });

  it('flushes a pending wheel crop when the session ends', async () => {
    vi.useFakeTimers();
    const c = session();
    updateDraftCrop(c, { x: 0.1, y: 0.1, w: 0.5, h: 0.5 });
    scheduleCropCommit(c);
    const ended = endSession(c);
    expect(c.geometrySession).toBeNull();
    await ended;
    expect(c.actions).toEqual(['Crop']);
    await vi.runAllTimersAsync();
    expect(c.actions).toEqual(['Crop']);
  });

  it('renders a new frame only when non-geometry edits change', () => {
    const c = session();
    const edits = neutralEdits() as Edits;
    refreshFrame(c, { ...edits, geometry: { ...edits.geometry, rotate: 90 } });
    expect(livePreview).toHaveBeenCalledTimes(1);
    refreshFrame(c, { ...edits, basic: { ...edits.basic, exposure_ev: 1 } });
    expect(livePreview).toHaveBeenCalledTimes(2);
    expect(vi.mocked(livePreview).mock.calls[1]?.[6]).toBe('geometry');
  });

  it.each([
    [60, 45],
    [-60, -45],
    [12.5, 12.5]
  ])('clamps a draft angle of %s to %s', (angle, want) => {
    const c = session();
    updateDraftAngle(c, angle);
    expect(c.geometrySession?.draftAngle).toBe(want);
  });

  it('flips a ratio crop between landscape and portrait and ignores square and free', async () => {
    const c = session();
    const sess = c.geometrySession;
    if (!sess) throw new Error('no session');
    await setAspect(c, { kind: 'ratio', num: 5, den: 4 });
    await flipAspect(c);
    expect(sess.draftAspect).toEqual({ kind: 'ratio', num: 4, den: 5 });
    expect(sess.draftCrop.w * 600).toBeLessThan(sess.draftCrop.h * 400);
    await setAspect(c, { kind: 'free' });
    await flipAspect(c);
    expect(sess.draftAspect).toEqual({ kind: 'free' });
  });

  it.each([
    ['an inset crop', { x: 0.1, y: 0.1, w: 0.5, h: 0.5 }, false],
    ['a crop dragged back to full', { x: -0.2, y: -0.1, w: 1.5, h: 1.3 }, true]
  ])('treats %s as default: %s', (_name, crop, want) => {
    const c = session();
    updateDraftCrop(c, { x: 0.1, y: 0.1, w: 0.5, h: 0.5 });
    updateDraftCrop(c, crop);
    const sess = c.geometrySession;
    if (!sess) throw new Error('no session');
    expect(cropIsDefault(sess)).toBe(want);
    expect(sess.userEditedCrop).toBe(!want);
  });

  it.each([
    ['right', 90, false, { x: 0.4, y: 0.1, w: 0.4, h: 0.3 }],
    ['left', 270, false, { x: 0.2, y: 0.6, w: 0.4, h: 0.3 }],
    ['right while flipped', 90, true, { x: 0.2, y: 0.6, w: 0.4, h: 0.3 }]
  ] as const)('turns the crop with a quarter turn %s', (_name, delta, flip, want) => {
    const c = session();
    const sess = c.geometrySession;
    if (!sess) throw new Error('no session');
    sess.draftFlipH = flip;
    sess.draftAspect = { kind: 'ratio', num: 3, den: 4 };
    updateDraftCrop(c, { x: 0.1, y: 0.2, w: 0.3, h: 0.4 });
    rotateStep(c, delta);
    for (const key of ['x', 'y', 'w', 'h'] as const) {
      expect(sess.draftCrop[key]).toBeCloseTo(want[key]);
    }
    expect(sess.draftAspect).toEqual({ kind: 'ratio', num: 4, den: 3 });
  });

  it('resets all geometry in one step and keeps develop edits', async () => {
    const c = session();
    c.edits.basic.exposure_ev = 1;
    rotateStep(c, 90);
    updateDraftAngle(c, 5);
    await commit(c, 'Straighten');
    await reset(c);
    await reset(c);
    expect(c.edits.geometry).toEqual(neutralGeometry());
    expect(c.edits.basic.exposure_ev).toBe(1);
    expect(c.geometrySession?.draftAngle).toBe(0);
    expect(c.actions).toEqual(['Rotate', 'Straighten', 'Reset Geometry']);
  });
});
