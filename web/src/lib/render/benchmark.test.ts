import { describe, expect, it, vi } from 'vitest';
import { BENCHMARK_EDITS, runBenchmark } from './benchmark';
import type { Renderer } from './dispatch';

function fakeRenderer(render: Renderer['render']): Renderer {
  return {
    adapter: () => 'fake',
    limits: () => ({ max_texture_dimension_2d: 8192, max_buffer_size: 1 << 30 }),
    set_synthetic_source: vi.fn(),
    drop_source: vi.fn(),
    set_source: vi.fn(),
    set_tile: vi.fn(),
    drop_tile: vi.fn(),
    set_raster: vi.fn(),
    drop_raster: vi.fn(),
    set_lut: vi.fn(),
    drop_lut: vi.fn(),
    set_dcp: vi.fn(),
    render
  };
}

describe('runBenchmark', () => {
  it('reports the median of the warm runs on a synthetic source', async () => {
    const durations = [500, 30, 10, 20];
    let clock = 0;
    let run = 0;
    const close = vi.fn();
    const renderer = fakeRenderer(async () => {
      clock += durations[run++] ?? 0;
      return { bitmap: { close }, width: 1600, height: 1066 };
    });

    const sample = await runBenchmark(renderer, 1600, () => clock);

    expect(sample).toEqual({ ms: 20, pixels: 1600 * 1066 });
    expect(renderer.set_synthetic_source).toHaveBeenCalledWith(1600);
    expect(renderer.drop_source).toHaveBeenCalledOnce();
    expect(close).toHaveBeenCalledTimes(durations.length);
    expect(JSON.parse(BENCHMARK_EDITS)).toHaveProperty('tone.highlights');
  });

  it('drops the synthetic source when a render fails', async () => {
    const renderer = fakeRenderer(async () => {
      throw new Error('device lost');
    });

    await expect(runBenchmark(renderer, 1600)).rejects.toThrow('device lost');
    expect(renderer.drop_source).toHaveBeenCalledOnce();
  });
});
