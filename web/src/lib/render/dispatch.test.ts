import { describe, expect, it, vi } from 'vitest';
import { createDispatcher, type Renderer } from './dispatch';

function fakeRenderer(): Renderer & { calls: string[] } {
  const calls: string[] = [];
  return {
    calls,
    adapter: () => 'fake adapter',
    set_source: (bytes) => {
      calls.push(`source:${bytes.length}`);
      return { width: 4, height: 2 };
    },
    set_raster: (id, width, height, bytes) =>
      void calls.push(`raster:${id}:${width}x${height}:${bytes.length}`),
    drop_raster: (id) => void calls.push(`drop-raster:${id}`),
    set_lut: (id, bytes) => void calls.push(`lut:${id}:${bytes.length}`),
    drop_lut: (id) => void calls.push(`drop-lut:${id}`),
    set_dcp: (bytes) => void calls.push(`dcp:${bytes ? bytes.length : 'none'}`),
    render: async (edits, view) => {
      calls.push(`render-start:${view}`);
      await new Promise((resolve) => setTimeout(resolve, 5));
      calls.push(`render-end:${edits}`);
      return { width: 4, height: 2 };
    }
  };
}

const canvas = {} as OffscreenCanvas;

describe('render dispatcher', () => {
  it('refuses calls before the renderer exists', async () => {
    const dispatch = createDispatcher(async () => fakeRenderer());
    await expect(dispatch({ op: 'dropLut', id: 'x' })).rejects.toThrow('not been initialised');
  });

  it('maps every call onto the wasm renderer', async () => {
    const renderer = fakeRenderer();
    const load = vi.fn(async () => renderer);
    const dispatch = createDispatcher(load);

    expect(await dispatch({ op: 'init', canvas })).toBe('fake adapter');
    expect(load).toHaveBeenCalledWith(canvas);
    expect(await dispatch({ op: 'setSource', bytes: new ArrayBuffer(6) })).toEqual({
      width: 4,
      height: 2
    });
    await dispatch({ op: 'setRaster', id: 'r', width: 2, height: 1, bytes: new ArrayBuffer(2) });
    await dispatch({ op: 'dropRaster', id: 'r' });
    await dispatch({ op: 'setLut', id: 'l', bytes: new ArrayBuffer(3) });
    await dispatch({ op: 'dropLut', id: 'l' });
    await dispatch({ op: 'setDcp', bytes: new ArrayBuffer(5) });
    await dispatch({ op: 'setDcp', bytes: null });

    expect(renderer.calls).toEqual([
      'source:6',
      'raster:r:2x1:2',
      'drop-raster:r',
      'lut:l:3',
      'drop-lut:l',
      'dcp:5',
      'dcp:none'
    ]);
  });

  it('runs calls one at a time so a render never overlaps another', async () => {
    const renderer = fakeRenderer();
    const dispatch = createDispatcher(async () => renderer);
    await dispatch({ op: 'init', canvas });

    const view = { max_edge: 64 };
    await Promise.all([
      dispatch({ op: 'render', edits: {} as never, view }),
      dispatch({ op: 'setSource', bytes: new ArrayBuffer(1) }),
      dispatch({ op: 'render', edits: {} as never, view })
    ]);

    expect(renderer.calls).toEqual([
      'render-start:{"max_edge":64}',
      'render-end:{}',
      'source:1',
      'render-start:{"max_edge":64}',
      'render-end:{}'
    ]);
  });

  it('keeps serving after a call fails', async () => {
    const renderer = fakeRenderer();
    renderer.set_lut = () => {
      throw new Error('bad cube');
    };
    const dispatch = createDispatcher(async () => renderer);
    await dispatch({ op: 'init', canvas });

    await expect(dispatch({ op: 'setLut', id: 'l', bytes: new ArrayBuffer(1) })).rejects.toThrow(
      'bad cube'
    );
    await dispatch({ op: 'dropLut', id: 'l' });
    expect(renderer.calls).toEqual(['drop-lut:l']);
  });
});
