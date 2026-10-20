import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BrushBuffer } from '$lib/utils/brush';

const mocks = vi.hoisted(() => ({
  fetchRaster: vi.fn(() =>
    Promise.resolve({ width: 4, height: 1, bytes: new Uint8Array([255, 170, 85, 0]) })
  )
}));

vi.mock('$lib/api/rasters', () => ({ fetchRaster: mocks.fetchRaster }));

import { DepthPreview, type DepthPreviewCtx } from '$lib/stores/editor/depthPreview';

const original: BrushBuffer = { width: 4, height: 1, bytes: new Uint8Array(4) };
const rebaked: BrushBuffer = { width: 4, height: 1, bytes: new Uint8Array(4).fill(9) };

let frames: FrameRequestCallback[] = [];

async function flushFrames(): Promise<void> {
  const pending = frames;
  frames = [];
  pending.forEach((cb) => cb(0));
  await vi.waitFor(() => expect(ctx.maskStroke).toBe('painting'));
}

let ctx: DepthPreviewCtx & { ended: number };

beforeEach(() => {
  frames = [];
  vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => frames.push(cb));
  vi.stubGlobal('cancelAnimationFrame', () => undefined);
  ctx = {
    brushBuffers: { c: original },
    brushBufferSource: { c: 'raster' },
    maskStroke: 'idle',
    ended: 0,
    endMaskStroke() {
      this.ended += 1;
    }
  };
});

afterEach(() => vi.unstubAllGlobals());

describe('DepthPreview', () => {
  it.each([
    ['restores the mask when no rebake landed', null, original],
    ['keeps the mask a rebake stored', rebaked, rebaked]
  ])('%s', async (_name, stored, want) => {
    const preview = new DepthPreview(ctx);
    preview.show({
      componentId: 'c',
      rasterId: 'raster',
      probRasterId: 'prob',
      range: { min: 0, max: 0.6, softness: 0 }
    });
    await flushFrames();
    expect(Array.from(ctx.brushBuffers.c?.bytes ?? [])).toEqual([255, 255, 0, 0]);
    if (stored) ctx.brushBuffers = { c: stored };
    preview.end();
    expect(ctx.brushBuffers.c).toBe(want);
    expect(ctx.brushBufferSource.c).toBe('raster');
    expect(ctx.ended).toBe(1);
  });
});
