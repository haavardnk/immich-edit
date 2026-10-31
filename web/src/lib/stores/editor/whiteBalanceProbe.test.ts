import { beforeEach, describe, expect, it, vi } from 'vitest';
import { neutralEdits, type WhiteBalance } from '#lib/types/edits.ts';

const mocks = vi.hoisted(() => ({ sampleWhiteBalance: vi.fn() }));

vi.mock('#lib/api/edits.ts', () => ({ sampleWhiteBalance: mocks.sampleWhiteBalance }));

import { WhiteBalanceProbe } from './whiteBalanceProbe.svelte';

const target = { assetId: 'asset-1', edits: neutralEdits() };

function point(u: number): { u: number; v: number; w: number; h: number } {
  return { u, v: 0.5, w: 0, h: 0 };
}

function deferred(): { promise: Promise<WhiteBalance>; resolve: (wb: WhiteBalance) => void } {
  let resolve: (wb: WhiteBalance) => void = () => undefined;
  const promise = new Promise<WhiteBalance>((r) => (resolve = r));
  return { promise, resolve };
}

describe('white balance probe', () => {
  beforeEach(() => {
    mocks.sampleWhiteBalance.mockReset();
  });

  it('keeps one probe in flight and streams the latest position', async () => {
    const first = deferred();
    const last = deferred();
    mocks.sampleWhiteBalance.mockReturnValueOnce(first.promise).mockReturnValueOnce(last.promise);
    const probe = new WhiteBalanceProbe(() => target);

    probe.probe(point(0.1));
    probe.probe(point(0.2));
    probe.probe(point(0.3));
    first.resolve({ mode: 'relative', temp: 4, tint: 1 });
    await vi.waitFor(() =>
      expect(probe.reading).toEqual({ wb: { mode: 'relative', temp: 4, tint: 1 }, error: null })
    );
    last.resolve({ mode: 'relative', temp: 9, tint: 2 });
    await vi.waitFor(() =>
      expect(probe.reading).toEqual({ wb: { mode: 'relative', temp: 9, tint: 2 }, error: null })
    );

    expect(mocks.sampleWhiteBalance.mock.calls.map((c) => c[1])).toEqual([point(0.1), point(0.3)]);
    expect(mocks.sampleWhiteBalance.mock.calls[0]?.[3]).toEqual({ silent: true });
  });

  it('shows a rejected probe as its message', async () => {
    mocks.sampleWhiteBalance.mockRejectedValue(new Error('No usable colour here'));
    const probe = new WhiteBalanceProbe(() => target);

    probe.probe(point(0.5));

    await vi.waitFor(() =>
      expect(probe.reading).toEqual({ wb: null, error: 'No usable colour here' })
    );
  });

  it('drops a reply that lands after clear', async () => {
    const reply = deferred();
    mocks.sampleWhiteBalance.mockReturnValueOnce(reply.promise);
    const probe = new WhiteBalanceProbe(() => target);

    probe.probe(point(0.5));
    probe.clear();
    reply.resolve({ mode: 'relative', temp: 4, tint: 1 });
    await reply.promise;
    await Promise.resolve();

    expect(probe.reading).toBeNull();
  });
});
