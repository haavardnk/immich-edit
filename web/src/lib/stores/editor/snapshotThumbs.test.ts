import type { EditSnapshot } from '#lib/api/snapshots.ts';
import { neutralEdits } from '#lib/types/edits.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({ livePreview: vi.fn() }));

vi.mock('#lib/api/preview.ts', () => ({ livePreview: mocks.livePreview }));

import { SnapshotThumbs } from '#lib/stores/editor/snapshotThumbs.svelte.ts';

function snapshot(hash: string): EditSnapshot {
  return {
    id: `id-${hash}`,
    name: hash,
    manifest_hash: hash,
    edits: neutralEdits(),
    created_at: '',
    updated_at: ''
  };
}

function deferred(): { promise: Promise<{ blob: Blob }>; resolve: () => void } {
  let resolve = (): void => undefined;
  const promise = new Promise<{ blob: Blob }>((done) => {
    resolve = () => done({ blob: new Blob(['x']) });
  });
  return { promise, resolve };
}

let created = 0;

beforeEach(() => {
  created = 0;
  vi.stubGlobal('URL', {
    createObjectURL: vi.fn(() => `blob:${++created}`),
    revokeObjectURL: vi.fn()
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
  mocks.livePreview.mockReset();
});

describe('SnapshotThumbs', () => {
  it('renders one thumbnail at a time on the reference lane and caches by hash', async () => {
    const first = deferred();
    const second = deferred();
    mocks.livePreview.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
    const thumbs = new SnapshotThumbs();

    thumbs.request('a1', [snapshot('h1'), snapshot('h2')]);
    thumbs.request('a1', [snapshot('h1'), snapshot('h2')]);
    expect(mocks.livePreview).toHaveBeenCalledTimes(1);
    expect(mocks.livePreview.mock.calls[0]?.[2]).toBe(160);
    expect(mocks.livePreview.mock.calls[0]?.[6]).toBe('reference');

    first.resolve();
    await vi.waitFor(() => expect(mocks.livePreview).toHaveBeenCalledTimes(2));
    expect(thumbs.url('h1')).toBe('blob:1');
    second.resolve();
    await vi.waitFor(() => expect(thumbs.url('h2')).toBe('blob:2'));

    thumbs.request('a1', [snapshot('h1'), snapshot('h2')]);
    expect(mocks.livePreview).toHaveBeenCalledTimes(2);
  });

  it('revokes every url and aborts the render on reset', async () => {
    const pending = deferred();
    mocks.livePreview
      .mockResolvedValueOnce({ blob: new Blob(['x']) })
      .mockReturnValueOnce(pending.promise);
    const thumbs = new SnapshotThumbs();

    thumbs.request('a1', [snapshot('h1'), snapshot('h2')]);
    await vi.waitFor(() => expect(mocks.livePreview).toHaveBeenCalledTimes(2));
    const signal = mocks.livePreview.mock.calls[1]?.[5] as AbortSignal;

    thumbs.reset();
    expect(signal.aborted).toBe(true);
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:1');
    expect(thumbs.url('h1')).toBeNull();

    pending.resolve();
    await Promise.resolve();
    expect(thumbs.url('h2')).toBeNull();
  });

  it('revokes the urls of snapshots that are no longer listed', async () => {
    mocks.livePreview.mockResolvedValue({ blob: new Blob(['x']) });
    const thumbs = new SnapshotThumbs();

    thumbs.request('a1', [snapshot('h1'), snapshot('h2')]);
    await vi.waitFor(() => expect(thumbs.url('h2')).toBe('blob:2'));

    thumbs.request('a1', [snapshot('h2')]);
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:1');
    expect(URL.revokeObjectURL).not.toHaveBeenCalledWith('blob:2');
    expect(thumbs.url('h1')).toBeNull();
    expect(thumbs.url('h2')).toBe('blob:2');
  });
});
