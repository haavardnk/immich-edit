import { livePreview } from '#lib/api/preview.ts';
import { neutralEdits, neutralGeometry, type Edits } from '#lib/types/edits.ts';
import type { UprightGuide } from '#lib/utils/upright.ts';
import { afterEach, describe, expect, it, vi } from 'vitest';

vi.mock('#lib/api/preview.ts', () => ({
  livePreview: vi.fn(() => new Promise<never>(() => undefined))
}));

import {
  endSession,
  flipStep,
  reset,
  resetTransform,
  rotateStep,
  startSession,
  syncSession,
  type GeometryCtx
} from './geometry.svelte';
import { addGuide, moveGuide, removeGuide } from './upright';

type TestCtx = GeometryCtx & { actions: string[] };

let open: TestCtx | null = null;

afterEach(async () => {
  if (open) await endSession(open);
  open = null;
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

function guides(c: TestCtx): UprightGuide[] {
  return c.geometrySession?.guides ?? [];
}

const LEFT: UprightGuide = { kind: 'vertical', from: [0.3, 0.9], to: [0.35, 0.1] };
const RIGHT: UprightGuide = { kind: 'vertical', from: [0.7, 0.9], to: [0.65, 0.1] };
const TOP: UprightGuide = { kind: 'horizontal', from: [0.1, 0.3], to: [0.9, 0.36] };

async function twoVerticals(): Promise<TestCtx> {
  const c = session();
  await addGuide(c, LEFT);
  await addGuide(c, RIGHT);
  return c;
}

describe('guided upright', () => {
  it('applies only once two guides exist', async () => {
    const c = session();
    await addGuide(c, LEFT);
    expect(c.actions).toEqual([]);
    expect(c.edits.geometry.perspective).toBeNull();
    await addGuide(c, RIGHT);
    expect(c.actions).toEqual(['Guided upright']);
    expect(c.edits.geometry.perspective?.vertical).toBeGreaterThan(0);
    expect(c.edits.geometry.crop).not.toBeNull();
  });

  it('ignores guides until the frame has loaded', async () => {
    const c = session(false);
    await addGuide(c, LEFT);
    await addGuide(c, RIGHT);
    expect(guides(c)).toEqual([]);
    expect(c.actions).toEqual([]);
  });

  it('re-solves when a guide moves', async () => {
    const c = await twoVerticals();
    const before = c.edits.geometry.perspective?.vertical ?? 0;
    await moveGuide(c, 1, { ...RIGHT, to: [0.6, 0.1] });
    expect(c.actions).toEqual(['Guided upright', 'Guided upright']);
    expect(c.edits.geometry.perspective?.vertical ?? 0).toBeGreaterThan(before);
  });

  it('adds shear from a guide in the other family', async () => {
    const c = await twoVerticals();
    await addGuide(c, TOP);
    expect(c.edits.geometry.perspective?.corners).not.toBeNull();
    expect(c.actions).toHaveLength(2);
  });

  it('keeps the applied correction when guides drop below two', async () => {
    const c = await twoVerticals();
    const geometry = c.edits.geometry;
    await removeGuide(c, 0);
    expect(guides(c)).toEqual([RIGHT]);
    expect(c.edits.geometry).toBe(geometry);
    expect(c.actions).toEqual(['Guided upright']);
  });

  it('re-solves when a guide is removed and two remain', async () => {
    const c = await twoVerticals();
    await addGuide(c, TOP);
    await removeGuide(c, 2);
    expect(guides(c)).toEqual([LEFT, RIGHT]);
    expect(c.edits.geometry.perspective?.corners).toBeNull();
    expect(c.actions).toEqual(['Guided upright', 'Guided upright', 'Guided upright']);
  });

  it('drops guides on a reset before anything was applied', async () => {
    const c = session();
    await addGuide(c, LEFT);
    await reset(c);
    expect(guides(c)).toEqual([]);
    expect(c.actions).toEqual([]);
  });

  it('keeps guides through an undo that leaves orientation alone', async () => {
    const c = await twoVerticals();
    syncSession(c, neutralGeometry());
    expect(guides(c)).toHaveLength(2);
    expect(c.geometrySession?.draftPerspective.vertical).toBe(0);
  });

  it.each([
    ['rotate', (c: TestCtx) => rotateStep(c, 90)],
    ['flip', (c: TestCtx) => flipStep(c, 'h')],
    ['reset', (c: TestCtx) => void reset(c)],
    ['a transform reset', (c: TestCtx) => void resetTransform(c)],
    ['an orientation undo', (c: TestCtx) => syncSession(c, { ...neutralGeometry(), rotate: 90 })]
  ])('drops guides on %s', async (_, act) => {
    const c = await twoVerticals();
    act(c);
    expect(guides(c)).toEqual([]);
  });
});
