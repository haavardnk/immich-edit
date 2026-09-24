import { describe, expect, it, vi } from 'vitest';
import { neutralEdits, type Edits } from '$lib/types/edits';

vi.mock('$lib/api/preview', () => ({
  livePreview: vi.fn(() => new Promise<never>(() => undefined))
}));

import {
  cancelSession,
  finishSession,
  sessionDirty,
  startSession,
  updateDraftAngle,
  type GeometryCtx
} from './geometry.svelte';

function ctx(): GeometryCtx & { commits: number } {
  const state = {
    assetId: 'asset-1',
    initialised: true,
    edits: neutralEdits() as Edits,
    geometrySession: null,
    error: null,
    commits: 0,
    clearView: () => undefined,
    onCommit: async () => {
      state.commits += 1;
    }
  };
  return state;
}

describe('geometry session', () => {
  it('is clean until the draft moves', () => {
    const c = ctx();
    startSession(c);
    const sess = c.geometrySession;
    if (!sess) throw new Error('no session');
    sess.srcW = 600;
    sess.srcH = 400;
    expect(sessionDirty(sess)).toBe(false);
    updateDraftAngle(c, 5);
    expect(sessionDirty(sess)).toBe(true);
  });

  it('cancel drops the draft without touching the edits', async () => {
    const c = ctx();
    startSession(c);
    const sess = c.geometrySession;
    updateDraftAngle(c, 5);
    cancelSession(c);
    expect(c.geometrySession).toBeNull();
    c.geometrySession = sess;
    await finishSession(c);
    expect(c.edits.geometry.rotate_angle).toBe(0);
    expect(c.commits).toBe(0);
  });

  it('finish writes the draft and commits once', async () => {
    const c = ctx();
    startSession(c);
    updateDraftAngle(c, 5);
    await finishSession(c);
    expect(c.geometrySession).toBeNull();
    expect(c.edits.geometry.rotate_angle).toBe(5);
    expect(c.commits).toBe(1);
  });
});
