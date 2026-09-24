import { describe, expect, it, vi } from 'vitest';
import type { ClientRenderer } from '$lib/render/client-renderer';
import type { RenderView, RenderedFrame } from '$lib/render/protocol';
import { neutralEdits, type Edits } from '$lib/types/edits';
import { BASE_SLOT, ClientLane, TILE_SLOT, type ClientJob } from './client-lane';

const SOURCE = { width: 8, height: 4, frame_width: 8, frame_height: 4, is_raw: false, model: '' };

function frame(label: number, close: () => void = () => {}): RenderedFrame {
  return {
    bitmap: { width: label, height: 1, close } as ImageBitmap,
    width: label,
    height: 1,
    source_w: 8,
    source_h: 4,
    timings: []
  };
}

function job(edge: number): ClientJob {
  return { edits: neutralEdits(), view: { max_edge: edge } };
}

function held() {
  const waiting: Array<(frame: RenderedFrame) => void> = [];
  const client = {
    inputs: async (_edits: Edits, maxEdge: number) => ({
      sensor_key: String(maxEdge > 0),
      rasters: [],
      lut: null
    }),
    loadSource: async () => SOURCE,
    prepare: async () => {},
    render: (_edits: Edits, view: RenderView) =>
      new Promise<RenderedFrame>((resolve) =>
        waiting.push((f) => resolve({ ...f, width: view.max_edge }))
      )
  };
  return { client: client as unknown as ClientRenderer, waiting };
}

async function flush(): Promise<void> {
  for (let i = 0; i < 10; i++) await Promise.resolve();
}

describe('client lane', () => {
  it('renders only the latest job queued behind an in-flight render', async () => {
    const { client, waiting } = held();
    const drawn: number[] = [];
    const lane = new ClientLane(client, BASE_SLOT, {
      assetId: () => 'a',
      onFrame: (_job, f) => drawn.push(f.width),
      onError: (err) => {
        throw err;
      },
      onIdle: () => {}
    });

    lane.submit(job(1));
    await flush();
    waiting.shift()?.(frame(0));
    await flush();
    lane.submit(job(2));
    lane.submit(job(3));
    lane.submit(job(4));
    await flush();
    waiting.shift()?.(frame(0));
    await flush();
    waiting.shift()?.(frame(0));
    await flush();

    expect(drawn).toEqual([1, 2, 4]);
    expect(lane.busy).toBe(false);
  });

  it('closes a frame that lands after a reset instead of drawing it', async () => {
    const { client, waiting } = held();
    const onFrame = vi.fn();
    const close = vi.fn();
    const lane = new ClientLane(client, BASE_SLOT, {
      assetId: () => 'a',
      onFrame,
      onError: () => {},
      onIdle: () => {}
    });

    lane.submit(job(1));
    await flush();
    waiting.shift()?.(frame(0));
    await flush();
    onFrame.mockClear();
    lane.submit(job(2));
    await flush();
    lane.reset();
    waiting.shift()?.(frame(0, close));
    await flush();

    expect(onFrame).not.toHaveBeenCalled();
    expect(close).toHaveBeenCalledOnce();
  });

  it('draws a tile only from the tile fetched for its roi', async () => {
    const loads: Array<{ roi: string; land: () => void }> = [];
    const views: RenderView[] = [];
    const client = {
      inputs: async () => ({ sensor_key: 'k', rasters: [], lut: null }),
      loadTile: (_a: string, _e: Edits, _m: number, roi: number[]) =>
        new Promise((resolve) => loads.push({ roi: roi.join(','), land: () => resolve(SOURCE) })),
      prepare: async () => {},
      render: async (_edits: Edits, view: RenderView) => {
        views.push(view);
        return frame(1);
      }
    } as unknown as ClientRenderer;
    const lane = new ClientLane(client, TILE_SLOT, {
      assetId: () => 'a',
      onFrame: () => {},
      onError: (err) => {
        throw err;
      },
      onIdle: () => {}
    });
    const tile = (roi: [number, number, number, number]): ClientJob => ({
      edits: neutralEdits(),
      view: { max_edge: 512, roi }
    });

    lane.submit(tile([0, 0, 0.5, 0.5]));
    await flush();
    expect(views).toEqual([]);
    loads[0]?.land();
    await flush();
    lane.submit(tile([0.5, 0, 0.5, 0.5]));
    await flush();

    expect(loads.map((l) => l.roi)).toEqual(['0,0,0.5,0.5', '0.5,0,0.5,0.5']);
    expect(views.map((v) => [v.roi?.join(','), v.tile])).toEqual([['0,0,0.5,0.5', true]]);
    expect(lane.busy).toBe(true);
  });
});
