import { beforeEach, describe, expect, it, vi } from 'vitest';
import { MONTHS_REFRESH_MS, MonthCountsStore } from './monthCounts.svelte';
import type { MonthCount } from '#lib/types/search.ts';

const searchMonths = vi.hoisted(() => vi.fn());

vi.mock('#lib/api/search.ts', () => ({ searchMonths }));

function flush(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

const july: MonthCount[] = [{ month: '2025-07', count: 26 }];

beforeEach(() => searchMonths.mockReset());

describe('MonthCountsStore', () => {
  it('fetches once per filter', async () => {
    searchMonths.mockResolvedValue({ months: july });
    const store = new MonthCountsStore();
    store.ensure({ favorite: true });
    store.ensure({ favorite: true });
    await flush();
    expect(searchMonths).toHaveBeenCalledOnce();
    expect(store.months).toEqual(july);
  });

  it('refetches when the filter changes and ignores the older answer', async () => {
    let answerOld = (_value: unknown): void => undefined;
    searchMonths
      .mockImplementationOnce(() => new Promise((resolve) => (answerOld = resolve)))
      .mockResolvedValueOnce({ months: [] });
    const store = new MonthCountsStore();
    store.ensure({});
    store.ensure({ favorite: true });
    await flush();
    answerOld({ months: july });
    await flush();
    expect(searchMonths).toHaveBeenCalledTimes(2);
    expect(store.months).toEqual([]);
  });

  it('retries a filter whose fetch failed', async () => {
    searchMonths.mockRejectedValueOnce(new Error('down')).mockResolvedValueOnce({ months: july });
    const store = new MonthCountsStore();
    store.ensure({});
    await flush();
    expect(store.months).toEqual([]);
    store.ensure({});
    await flush();
    expect(store.months).toEqual(july);
  });

  it('treats a malformed answer as no months', async () => {
    searchMonths.mockResolvedValue({});
    const store = new MonthCountsStore();
    store.ensure({});
    await flush();
    expect(store.months).toEqual([]);
  });

  it('refreshes a watched filter once after a burst of changes and keeps the old months meanwhile', async () => {
    vi.useFakeTimers();
    try {
      let answer = (_value: unknown): void => undefined;
      searchMonths
        .mockResolvedValueOnce({ months: july })
        .mockImplementationOnce(() => new Promise((resolve) => (answer = resolve)));
      const store = new MonthCountsStore();
      const stop = store.watch({});
      await vi.runAllTimersAsync();
      store.invalidate();
      store.invalidate();
      await vi.advanceTimersByTimeAsync(MONTHS_REFRESH_MS);
      expect(searchMonths).toHaveBeenCalledTimes(2);
      expect(store.months).toEqual(july);
      answer({ months: [{ month: '2025-08', count: 3 }] });
      await vi.runAllTimersAsync();
      expect(store.months).toEqual([{ month: '2025-08', count: 3 }]);
      stop();
    } finally {
      vi.useRealTimers();
    }
  });

  it('refreshes an unwatched filter on its next use', async () => {
    searchMonths.mockResolvedValue({ months: july });
    const store = new MonthCountsStore();
    store.ensure({});
    await flush();
    store.invalidate();
    expect(searchMonths).toHaveBeenCalledOnce();
    store.ensure({});
    expect(store.months).toEqual(july);
    expect(searchMonths).toHaveBeenCalledTimes(2);
  });
});
