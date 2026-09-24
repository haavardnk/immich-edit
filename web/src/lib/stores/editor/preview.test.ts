import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { RenderLane } from '$lib/api/preview';
import type { RenderView, RenderedFrame } from '$lib/render/protocol';
import { neutralEdits, type Edits } from '$lib/types/edits';
import type { Roi } from '$lib/utils/view-geometry';

type Render = {
  lane: RenderLane;
  maxEdge: number;
  roi: Roi | undefined;
  open: boolean;
  land: () => void;
};

const renders = vi.hoisted((): Render[] => []);

vi.mock('$lib/api/preview', async (original) => ({
  ...(await original<typeof import('$lib/api/preview')>()),
  livePreview: vi.fn(
    (
      _assetId: string,
      _edits: unknown,
      maxEdge: number,
      _mode: unknown,
      _proof: unknown,
      signal: AbortSignal,
      lane: RenderLane,
      roi?: Roi
    ) =>
      new Promise((resolve, reject) => {
        const render: Render = {
          lane,
          maxEdge,
          roi,
          open: true,
          land: () => {
            render.open = false;
            resolve({ blob: new Blob(), metaId: null });
          }
        };
        signal.addEventListener('abort', () => {
          render.open = false;
          reject(new DOMException('aborted', 'AbortError'));
        });
        renders.push(render);
      })
  )
}));
vi.mock('$lib/utils/object-url', () => ({ makeObjectUrl: () => 'blob:render', revoke: () => {} }));

const browser = vi.hoisted(() => ({
  current: null as ClientRenderer | null,
  fail: vi.fn()
}));

vi.mock('$lib/stores/renderer.svelte', () => ({
  renderer: {
    undecided: false,
    get current() {
      return browser.current;
    },
    start: async () => browser.current,
    fail: browser.fail,
    recordRender: () => {}
  }
}));

import type { ClientRenderer } from '$lib/render/client-renderer';
import { PreviewEngine, type PreviewCtx, type ViewSnapshot } from './preview.svelte';

const SNAP: ViewSnapshot = {
  frame: { left: 40, top: 50, width: 1200, height: 800 },
  viewW: 1280,
  viewH: 900,
  dpr: 2
};
const DRAG_EDGE = 1280;
const SETTLED_EDGE = 2400;

class DecodedImage {
  src = '';
  naturalWidth = SETTLED_EDGE;
  naturalHeight = 1600;
  decode(): Promise<void> {
    return Promise.resolve();
  }
}

function context(): PreviewCtx {
  return {
    assetId: 'asset-1',
    initialised: true,
    edits: neutralEdits(),
    meta: null,
    previewUrl: null,
    previewFrame: null,
    originalUrl: null,
    viewUrl: null,
    viewRoi: null,
    viewNat: null,
    pending: false,
    error: null,
    splitMode: false,
    showingOriginal: false,
    bypassedSection: null,
    geometrySession: null,
    maskPreviewLayerId: null,
    colorPicker: null,
    proofSpace: 'srgb',
    gamutWarn: false
  };
}

function lanes(): string[] {
  return renders.map((r) => `${r.lane}@${r.maxEdge}`);
}

function openRenders(): number {
  return renders.filter((r) => r.open).length;
}

async function landLatest(): Promise<void> {
  const latest = renders.at(-1);
  if (!latest) throw new Error('nothing was rendered');
  latest.land();
  await vi.advanceTimersByTimeAsync(0);
}

describe('preview lanes during a slider drag', () => {
  let engine: PreviewEngine;

  beforeEach(async () => {
    vi.useFakeTimers();
    vi.stubGlobal('Image', DecodedImage);
    renders.length = 0;
    engine = new PreviewEngine(context());
    engine.onViewChange(SNAP);
    await vi.advanceTimersByTimeAsync(1000);
    await landLatest();
    renders.length = 0;
  });

  afterEach(() => {
    engine.reset();
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  it('renders one base at 1x the viewport per tick and no view lane', async () => {
    engine.beginDrag();
    for (let tick = 0; tick < 4; tick++) {
      engine.live();
      await vi.advanceTimersByTimeAsync(400);
      expect(openRenders()).toBe(1);
    }
    expect(renders.every((r) => r.lane === 'base' && r.maxEdge === DRAG_EDGE)).toBe(true);
  });

  it('renders the release at full DPR and holds the view lane for the base', async () => {
    engine.beginDrag();
    engine.live();
    engine.endDrag();
    engine.live();
    await vi.advanceTimersByTimeAsync(1000);
    expect(lanes()).toEqual([`base@${DRAG_EDGE}`, `base@${SETTLED_EDGE}`]);
    expect(openRenders()).toBe(1);

    await landLatest();
    expect(lanes().at(-1)).toBe(`roi@${SETTLED_EDGE}`);
    expect(openRenders()).toBe(1);
  });

  it('restores full DPR when the drag ends without a commit', async () => {
    engine.beginDrag();
    engine.live();
    engine.endDrag();
    await vi.advanceTimersByTimeAsync(0);
    expect(lanes()).toEqual([`base@${DRAG_EDGE}`, `base@${SETTLED_EDGE}`]);
  });

  it('settles once when the commit lands before the pointer is released', async () => {
    engine.beginDrag();
    engine.live();
    engine.live();
    engine.endDrag();
    await vi.advanceTimersByTimeAsync(0);
    expect(lanes()).toEqual([`base@${DRAG_EDGE}`, `base@${DRAG_EDGE}`, `base@${SETTLED_EDGE}`]);
  });

  it('renders nothing for a press that never moved the value', async () => {
    engine.beginDrag();
    engine.endDrag();
    await vi.advanceTimersByTimeAsync(1000);
    expect(renders).toEqual([]);
  });

  it('holds the view lane when the frame moves mid-drag', async () => {
    engine.beginDrag();
    engine.live();
    engine.onViewChange({ ...SNAP, frame: { ...SNAP.frame, height: 801 } });
    await vi.advanceTimersByTimeAsync(400);
    await landLatest();
    await vi.advanceTimersByTimeAsync(400);
    expect(lanes()).toEqual([`base@${DRAG_EDGE}`]);
  });

  it('renders the view lane for a frame that moved during a still press', async () => {
    engine.beginDrag();
    engine.onViewChange({ ...SNAP, frame: { left: -600, top: -400, width: 2400, height: 1600 } });
    engine.endDrag();
    await vi.advanceTimersByTimeAsync(1000);
    expect(lanes().map((lane) => lane.split('@')[0])).toEqual(['roi']);
  });
});

function browserFrame(): RenderedFrame {
  const bins = {
    r: new Uint32Array(1),
    g: new Uint32Array(1),
    b: new Uint32Array(1),
    l: new Uint32Array(1)
  };
  return {
    bitmap: { width: 2400, height: 1600, close: () => {} } as ImageBitmap,
    width: 2400,
    height: 1600,
    source_w: 6000,
    source_h: 4000,
    histogram: bins,
    timings: []
  };
}

function fakeClient(render: () => Promise<RenderedFrame>) {
  const sources: number[] = [];
  const views: RenderView[] = [];
  const client = {
    inputs: async (edits: Edits) => ({
      sensor_key: String(edits.detail.luma_nr_amount),
      rasters: [],
      lut: null
    }),
    loadSource: async (_assetId: string, edits: Edits) => {
      sources.push(edits.detail.luma_nr_amount);
      return {
        width: 2400,
        height: 1600,
        frame_width: 6000,
        frame_height: 4000,
        is_raw: true,
        model: 'x'
      };
    },
    prepare: async () => {},
    render: (_edits: Edits, view: RenderView) => {
      views.push(view);
      return render();
    }
  };
  return { client: client as unknown as ClientRenderer, sources, views };
}

describe('preview lanes with the browser renderer', () => {
  let engine: PreviewEngine;
  let ctx: PreviewCtx;

  beforeEach(() => {
    vi.useFakeTimers();
    vi.stubGlobal('Image', DecodedImage);
    renders.length = 0;
    browser.fail.mockClear();
    ctx = context();
    engine = new PreviewEngine(ctx);
    engine.onViewChange(SNAP);
  });

  afterEach(() => {
    engine.reset();
    browser.current = null;
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  it('renders display edits locally and fetches a source only for sensor edits', async () => {
    const fake = fakeClient(async () => browserFrame());
    browser.current = fake.client;

    engine.live();
    await vi.advanceTimersByTimeAsync(1000);
    ctx.edits = { ...ctx.edits, basic: { ...ctx.edits.basic, exposure_ev: 1 } };
    engine.live();
    await vi.advanceTimersByTimeAsync(1000);
    ctx.edits = { ...ctx.edits, detail: { ...ctx.edits.detail, luma_nr_amount: 30 } };
    engine.live();
    await vi.advanceTimersByTimeAsync(1000);

    expect(renders).toEqual([]);
    expect(fake.sources).toEqual([0, 30]);
    expect(fake.views.length).toBe(4);
    expect(fake.views.every((view) => view.max_edge === SETTLED_EDGE && view.histogram)).toBe(true);
    expect(ctx.previewFrame?.bitmap.width).toBe(2400);
    expect(ctx.meta?.renderer).toBe('browser');
  });

  it('falls back to the server lane when the browser render fails', async () => {
    const fake = fakeClient(async () => {
      throw new Error('device lost');
    });
    browser.current = fake.client;

    engine.live();
    await vi.advanceTimersByTimeAsync(0);

    expect(browser.fail).toHaveBeenCalledOnce();
    expect(lanes()).toEqual([`base@${SETTLED_EDGE}`]);
  });
});
