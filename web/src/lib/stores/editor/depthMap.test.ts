import { beforeEach, describe, expect, it, vi } from 'vitest';
import { DepthMapState } from './depthMap.svelte';

const { generateDepth } = vi.hoisted(() => ({ generateDepth: vi.fn() }));
vi.mock('$lib/api/depth', () => ({ generateDepth }));

describe('depth map generation', () => {
  beforeEach(() => {
    generateDepth.mockReset();
  });

  it('applies the new raster and clears the busy state', async () => {
    generateDepth.mockResolvedValue({ raster_id: 'depth-1' });
    const state = new DepthMapState();
    const apply = vi.fn(async () => true);
    const run = state.generate('a', apply);
    expect(state.busy('a')).toBe(true);
    expect(state.busy('b')).toBe(false);
    await run;
    expect(apply).toHaveBeenCalledWith('depth-1');
    expect(state.busy('a')).toBe(false);
    expect(state.take('a')).toBeNull();
  });

  it('keeps a failure with the photo it belongs to', async () => {
    generateDepth.mockRejectedValue(new Error('no model installed for depth'));
    const state = new DepthMapState();
    await state.generate('a', async () => true);
    expect(state.error('a')).toBe('no model installed for depth');
    expect(state.error('b')).toBeNull();
  });

  it('runs photos independently and holds a result for a photo that was left', async () => {
    const finish: Record<string, (value: { raster_id: string }) => void> = {};
    generateDepth.mockImplementation(
      (id: string) => new Promise((resolve) => (finish[id] = resolve))
    );
    const state = new DepthMapState();
    const left = state.generate('a', async () => false);
    await state.generate('a', async () => true);
    const open = state.generate('b', async () => true);
    expect(generateDepth).toHaveBeenCalledTimes(2);
    expect(state.busy('a') && state.busy('b')).toBe(true);
    finish.a?.({ raster_id: 'depth-a' });
    finish.b?.({ raster_id: 'depth-b' });
    await Promise.all([left, open]);
    expect(state.take('b')).toBeNull();
    expect(state.take('a')).toBe('depth-a');
    expect(state.take('a')).toBeNull();
  });
});
