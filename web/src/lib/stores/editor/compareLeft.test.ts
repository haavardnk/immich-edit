import type { EditSnapshot } from '#lib/api/snapshots.ts';
import { ORIGINAL_SOURCE, type CompareMode, type CompareSource } from '#lib/editor/compare.ts';
import { neutralEdits, originalPreviewEdits, type Edits } from '#lib/types/edits.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

type Render = { assetId: string; edits: Edits; edge: number; lane: string };

const mocks = vi.hoisted(() => ({ renders: [] as Render[] }));

vi.mock('#lib/api/preview.ts', () => ({
  livePreview: vi.fn((assetId: string, edits: Edits, edge: number, ...rest: unknown[]) => {
    mocks.renders.push({ assetId, edits, edge, lane: String(rest[3]) });
    return Promise.resolve({ blob: new Blob(), metaId: null });
  })
}));
vi.mock('#lib/utils/objectUrl.ts', () => ({
  makeObjectUrl: () => 'blob:left',
  revoke: () => {}
}));

import { CompareLeft, type CompareLeftCtx } from '#lib/stores/editor/compareLeft.svelte.ts';

function withExposure(exposure: number): Edits {
  const edits = neutralEdits();
  edits.basic.exposure_ev = exposure;
  return edits;
}

const SNAPSHOT: EditSnapshot = {
  id: 'snap-1',
  name: 'Warm v1',
  manifest_hash: 'hash-1',
  edits: withExposure(0.7),
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z'
};

type View = { mode: CompareMode; source: CompareSource };

function context(view: View): CompareLeftCtx {
  return {
    assetId: 'asset-1',
    edits: withExposure(1.5),
    compare: view,
    compareUrl: null,
    snapshots: { items: [SNAPSHOT] }
  };
}

describe('CompareLeft', () => {
  let view: View;
  let ctx: CompareLeftCtx;
  let left: CompareLeft;
  let edge = 1600;

  beforeEach(() => {
    mocks.renders.length = 0;
    edge = 1600;
    view = { mode: 'side', source: ORIGINAL_SOURCE };
    ctx = context(view);
    left = new CompareLeft(ctx, { proof: () => ({}) as never, edge: () => edge });
  });

  afterEach(() => left.drop());

  async function settle(): Promise<void> {
    await vi.waitFor(() => expect(ctx.compareUrl).toBe('blob:left'));
  }

  it('renders the unedited current asset for the original', async () => {
    left.refresh();
    await settle();
    expect(mocks.renders).toEqual([
      {
        assetId: 'asset-1',
        edits: originalPreviewEdits(withExposure(1.5)),
        edge: 1600,
        lane: 'original'
      }
    ]);
  });

  it('renders the snapshot edits on the current asset', async () => {
    view.source = { kind: 'snapshot', id: 'snap-1' };
    left.refresh();
    await settle();
    expect(mocks.renders).toEqual([
      { assetId: 'asset-1', edits: SNAPSHOT.edits, edge: 1600, lane: 'original' }
    ]);
  });

  it('skips a refresh for the same source and edge', async () => {
    view.source = { kind: 'snapshot', id: 'snap-1' };
    left.refresh();
    await settle();
    ctx.edits = withExposure(3);
    left.refresh();
    expect(mocks.renders).toHaveLength(1);
  });

  it('re-renders the original when the geometry changes', async () => {
    left.refresh();
    await settle();
    ctx.edits = { ...ctx.edits, geometry: { ...ctx.edits.geometry, rotate: 90 } };
    left.refresh();
    await vi.waitFor(() => expect(mocks.renders).toHaveLength(2));
  });

  it('renders nothing while compare is off or for a missing snapshot', () => {
    view.mode = 'off';
    left.refresh();
    view.mode = 'split';
    view.source = { kind: 'snapshot', id: 'gone' };
    left.refresh();
    expect(mocks.renders).toEqual([]);
  });
});
