import { editsToManifest } from '#lib/edits/manifest.ts';
import { neutralEdits, type Edits } from '#lib/types/edits.ts';
import type { ViewerJob } from '#lib/viewer/jobs.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

type Call = {
  assetId: string;
  edits: Edits;
  edge: number;
  lane: string;
  roi: unknown;
  signal: AbortSignal;
  settle: (ok: boolean) => void;
};

const mocks = vi.hoisted(() => ({
  calls: [] as Call[],
  revoked: [] as string[],
  getEdits: vi.fn(),
  urls: 0
}));

vi.mock('#lib/api/preview.ts', () => ({
  livePreview: vi.fn(
    (
      assetId: string,
      edits: Edits,
      edge: number,
      _mode: unknown,
      _proof: unknown,
      signal: AbortSignal,
      lane: string,
      roi: unknown
    ) =>
      new Promise((resolve, reject) => {
        signal.addEventListener('abort', () => reject(new DOMException('aborted', 'AbortError')));
        mocks.calls.push({
          assetId,
          edits,
          edge,
          lane,
          roi,
          signal,
          settle: (ok) =>
            ok
              ? resolve({ blob: new Blob(), metaId: null, timing: null })
              : reject(new Error('render failed'))
        });
      })
  )
}));
vi.mock('#lib/api/edits.ts', () => ({ getEdits: mocks.getEdits }));
vi.mock('#lib/utils/objectUrl.ts', () => ({
  makeObjectUrl: () => `blob:${(mocks.urls += 1)}`,
  revoke: (url: string | null) => {
    if (url) mocks.revoked.push(url);
  }
}));

import { ViewerRenders } from '#lib/viewer/renders.svelte.ts';

const PROOF = { colorSpace: 'srgb' as const, gamutWarn: false, clipWarn: false };

function job(patch: Partial<ViewerJob>): ViewerJob {
  return {
    slot: 'base',
    key: 'base|1|2560',
    assetId: 'a1',
    edits: neutralEdits(),
    edge: 2560,
    fullEdge: 2560,
    proof: PROOF,
    roi: null,
    revision: 1,
    target: 'a1',
    patient: false,
    ...patch
  };
}

const flush = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

function call(index: number): Call {
  const found = mocks.calls[index];
  if (!found) throw new Error(`no render ${index}`);
  return found;
}

describe('ViewerRenders', () => {
  let renders: ViewerRenders;

  beforeEach(() => {
    mocks.calls = [];
    mocks.revoked = [];
    mocks.urls = 0;
    mocks.getEdits.mockReset();
    vi.stubGlobal(
      'Image',
      class {
        src = '';
        naturalWidth = 2560;
        naturalHeight = 1707;
        decode(): Promise<void> {
          return Promise.resolve();
        }
      }
    );
    renders = new ViewerRenders();
  });

  afterEach(() => {
    renders.dispose();
    vi.unstubAllGlobals();
  });

  it('renders one job at a time on the viewer lane', async () => {
    const tile = job({ slot: 'tile', key: 'tile|1|roi', roi: [0.25, 0.25, 0.5, 0.5], edge: 1400 });
    renders.plan([job({}), tile]);
    expect(mocks.calls.map((c) => [c.lane, c.roi])).toEqual([['viewer', undefined]]);
    call(0).settle(true);
    await flush();
    expect(renders.base).toMatchObject({ key: 'base|1|2560', url: 'blob:1', w: 2560, h: 1707 });
    expect(call(1).roi).toEqual([0.25, 0.25, 0.5, 0.5]);
    call(1).settle(true);
    await flush();
    expect(renders.tile?.fullEdge).toBe(Math.round((2560 * 2560) / 1400));
  });

  it('aborts a stale render for the newest revision', async () => {
    renders.plan([job({})]);
    renders.plan([job({ key: 'base|2|2560', revision: 2 })]);
    expect(call(0).signal.aborted).toBe(true);
    await flush();
    call(1).settle(true);
    await flush();
    expect(renders.base?.revision).toBe(2);
    expect(mocks.calls).toHaveLength(2);
  });

  it('lets a dragging base land before the next one starts', async () => {
    renders.plan([job({ key: 'base|1|1600', patient: true })]);
    renders.plan([job({ key: 'base|2|1600', revision: 2, patient: true })]);
    expect(call(0).signal.aborted).toBe(false);
    expect(mocks.calls).toHaveLength(1);
    call(0).settle(true);
    await flush();
    expect(renders.base?.revision).toBe(1);
    expect(call(1).signal.aborted).toBe(false);
  });

  it('revokes the image a new render replaces', async () => {
    renders.plan([job({})]);
    call(0).settle(true);
    await flush();
    renders.plan([job({ key: 'base|2|2560', revision: 2 })]);
    call(1).settle(true);
    await flush();
    expect(renders.base?.url).toBe('blob:2');
    expect(mocks.revoked).toEqual(['blob:1']);
  });

  it('does not retry a failed render', async () => {
    renders.plan([job({})]);
    call(0).settle(false);
    await flush();
    renders.plan([job({})]);
    expect(mocks.calls).toHaveLength(1);
    expect(renders.base).toBeNull();
  });

  it('renders a version from its saved edits once', async () => {
    const saved = neutralEdits();
    saved.basic.exposure_ev = 0.8;
    mocks.getEdits.mockResolvedValue({ manifest: editsToManifest(saved) });
    const left = job({ slot: 'left', key: 'left|v2|1', assetId: 'v2', edits: null });
    renders.plan([left]);
    await flush();
    expect(call(0).assetId).toBe('v2');
    expect(call(0).edits.basic.exposure_ev).toBe(0.8);
    call(0).settle(true);
    await flush();
    renders.plan([{ ...left, key: 'left|v2|2' }]);
    await flush();
    expect(mocks.getEdits).toHaveBeenCalledTimes(1);
    expect(renders.left?.key).toBe('left|v2|1');
  });
});
