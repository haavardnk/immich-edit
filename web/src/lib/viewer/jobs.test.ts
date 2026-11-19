import { neutralEdits } from '#lib/types/edits.ts';
import {
  baseEdge,
  landedFullEdge,
  viewerJobs,
  type JobInput,
  type LandedJob,
  type ViewerJob
} from '#lib/viewer/jobs.ts';
import type { ViewerRender } from '#lib/viewer/protocol.ts';
import { describe, expect, it } from 'vitest';

function render(patch: Partial<ViewerRender> = {}): ViewerRender {
  return {
    assetId: 'a1',
    name: 'IMG_1.CR3',
    edits: neutralEdits(),
    dragging: false,
    proofSpace: 'srgb',
    gamutWarn: false,
    clipWarn: false,
    compare: null,
    ...patch
  };
}

function input(patch: Partial<JobInput> = {}): JobInput {
  return {
    render: render(),
    revision: 1,
    pane: { w: 1200, h: 800 },
    dpr: 2,
    wide: false,
    srcLong: 6000,
    frame: null,
    base: null,
    tile: null,
    ...patch
  };
}

function landed(job: ViewerJob, w: number, h: number): LandedJob {
  return { ...job, url: `blob:${job.key}`, w, h, fullEdge: landedFullEdge(job, w, h) };
}

describe('baseEdge', () => {
  it('quantises the pane in device pixels inside the edge bounds', () => {
    expect(baseEdge({ w: 1200, h: 800 }, 2, 6000)).toBe(2560);
    expect(baseEdge({ w: 400, h: 300 }, 1, 6000)).toBe(1600);
    expect(baseEdge({ w: 3000, h: 2000 }, 2, 9000)).toBe(4096);
  });

  it('never asks for more than the source', () => {
    expect(baseEdge({ w: 1200, h: 800 }, 2, 2000)).toBe(2000);
    expect(baseEdge({ w: 1200, h: 800 }, 2, Number.POSITIVE_INFINITY)).toBe(2560);
  });
});

describe('viewerJobs', () => {
  it('waits for a measured pane', () => {
    expect(viewerJobs(input({ pane: { w: 0, h: 0 } }))).toEqual([]);
  });

  it('renders the base on the viewer proof', () => {
    const [job] = viewerJobs(input({ wide: true, render: render({ clipWarn: true }) }));
    expect(job).toMatchObject({
      slot: 'base',
      key: 'base|1|2560',
      edge: 2560,
      patient: false,
      proof: { colorSpace: 'displayp3', gamutWarn: false, clipWarn: true }
    });
  });

  it('drops to a small patient base while dragging and skips tiles', () => {
    const jobs = viewerJobs(input({ render: render({ dragging: true }) }));
    expect(jobs.map((job) => [job.slot, job.edge, job.patient])).toEqual([['base', 1600, true]]);
  });

  it('keeps a landed base that already covers the revision', () => {
    const [job] = viewerJobs(input());
    const base = landed(job!, 2560, 1707);
    expect(viewerJobs(input({ base, pane: { w: 1100, h: 700 } }))).toEqual([]);
  });

  it('renders the compare left at the base edge', () => {
    const left = { assetId: 'a1', edits: neutralEdits(), key: 'snapshot:s1:h1' };
    const compare = { mode: 'split' as const, left, labels: { left: 'Snap', right: 'Current' } };
    const jobs = viewerJobs(input({ render: render({ compare }) }));
    expect(jobs.map((job) => job.slot)).toEqual(['base', 'left']);
    expect(jobs[1]).toMatchObject({ edge: 2560, edits: left.edits });
    expect(jobs[1]?.target).toContain('a1|snapshot:s1:h1|');
  });

  it('fetches saved edits for a version left', () => {
    const left = { assetId: 'v2', edits: null, key: 'version:v2' };
    const compare = { mode: 'side' as const, left, labels: { left: 'Version', right: 'Current' } };
    const jobs = viewerJobs(input({ render: render({ compare }) }));
    expect(jobs[1]).toMatchObject({ slot: 'left', assetId: 'v2', edits: null });
  });

  it('adds a tile once the zoomed frame outgrows the base', () => {
    const [job] = viewerJobs(input());
    const base = landed(job!, 2560, 1707);
    const frame = { left: -2400, top: -1600, width: 6000, height: 4000 };
    const [tile] = viewerJobs(input({ base, frame }));
    expect(tile?.slot).toBe('tile');
    expect(tile?.roi).not.toBeNull();
    expect(tile?.roi?.[2]).toBeLessThan(1);
    const covered = landed(tile!, tile!.edge, tile!.edge);
    expect(viewerJobs(input({ base, frame, tile: covered }))).toEqual([]);
    expect(viewerJobs(input({ base, frame, tile: covered, revision: 2 }))[1]?.slot).toBe('tile');
  });

  it('needs no tile at fit', () => {
    const [job] = viewerJobs(input());
    const base = landed(job!, 2560, 1707);
    const frame = { left: 0, top: 0, width: 1200, height: 800 };
    expect(viewerJobs(input({ base, frame }))).toEqual([]);
  });
});
