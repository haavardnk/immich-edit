import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { gridHost } from '#lib/commands/grid/gridHost.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { browseControls } from '#lib/stores/browseControls.svelte.ts';
import { monthCounts } from '#lib/stores/monthCounts.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { DateScrub, KEY_IDLE_MS, type ScrubFeed } from './dateScrub.svelte';
import { gridViewport } from './gridViewport.svelte';

const searchFirst = vi.hoisted(() => vi.fn());

vi.mock('#lib/api/search.ts', () => ({ searchFirst, searchMonths: vi.fn() }));

function photo(id: string, fileCreatedAt: string): AssetSummary {
  return { id, fileCreatedAt, exifInfo: null } as AssetSummary;
}

const scrollTo = vi.fn();
let detach = (): void => {};

function feed(hasPrevious = false): ScrubFeed & { jumpTo: ReturnType<typeof vi.fn> } {
  return {
    hasPrevious,
    currentFilter: () => ({ favorite: true }),
    jumpTo: vi.fn(async () => true)
  };
}

beforeEach(() => {
  browseControls.reset();
  monthCounts.months = [
    { month: '2026-10', count: 2 },
    { month: '2025-07', count: 2 },
    { month: '2024-01', count: 4 }
  ];
  browsing.set([
    photo('a', '2026-10-05T00:00:00Z'),
    photo('b', '2026-10-01T00:00:00Z'),
    photo('c', '2025-07-04T00:00:00Z'),
    photo('d', '2025-07-01T00:00:00Z')
  ]);
  detach = gridHost.attach({
    items: () => [],
    selectingAll: () => false,
    selectAll: async () => true,
    cancelSelectAll: () => {},
    move: () => {},
    reveal: () => {},
    scrollTo,
    focusBulkBar: () => {},
    holdAnchor: () => () => {}
  });
});

afterEach(() => {
  detach();
  scrollTo.mockReset();
  searchFirst.mockReset();
  toasts.items = [];
  gridViewport.setTop(null);
});

describe('DateScrub', () => {
  it('scrolls through a single month by the drag position', async () => {
    monthCounts.months = [{ month: '2026-10', count: 4 }];
    browsing.set([
      photo('a', '2026-10-20T00:00:00Z'),
      photo('b', '2026-10-15T00:00:00Z'),
      photo('c', '2026-10-10T00:00:00Z'),
      photo('d', '2026-10-05T00:00:00Z')
    ]);
    const source = feed();
    const scrub = new DateScrub(() => source);
    scrub.begin(0.1);
    expect(scrollTo).toHaveBeenLastCalledWith('a');
    scrub.drag(0.6);
    expect(scrollTo).toHaveBeenLastCalledWith('c');
    scrub.drag(0.95);
    expect(scrollTo).toHaveBeenLastCalledWith('d');
    scrub.end(false);
    await vi.waitFor(() => expect(scrollTo).toHaveBeenCalledTimes(4));
    expect(scrollTo).toHaveBeenLastCalledWith('d');
    expect(searchFirst).not.toHaveBeenCalled();
    expect(source.jumpTo).not.toHaveBeenCalled();
  });

  it('pages a month in from its furthest loaded photo when the target is not loaded', async () => {
    monthCounts.months = [{ month: '2026-10', count: 8 }];
    browsing.set([photo('a', '2026-10-20T00:00:00Z'), photo('b', '2026-10-19T00:00:00Z')]);
    const source = feed();
    source.jumpTo.mockImplementation(async (id: string) => {
      if (id === 'b') {
        browsing.set(
          ['a', 'b', 'c', 'd', 'e', 'f', 'g'].map((name, at) =>
            photo(name, `2026-10-${20 - at}T00:00:00Z`)
          )
        );
      }
      return true;
    });
    const scrub = new DateScrub(() => source);
    expect(await scrub.go(0, 6)).toBe(true);
    expect(source.jumpTo).toHaveBeenCalledOnce();
    expect(source.jumpTo).toHaveBeenCalledWith('b', { failure: 'Jump to date' });
    expect(scrollTo).toHaveBeenLastCalledWith('g');
    expect(searchFirst).not.toHaveBeenCalled();
  });

  it('marks the month of the top visible photo', () => {
    const scrub = new DateScrub(() => feed());
    gridViewport.setTop('d');
    expect(scrub.visible).toEqual({ index: 1, within: 0.5 });
    expect(scrub.markAt).toBe(0.375);
    expect(scrub.index).toBe(1);
  });

  it('scrolls live to a loaded month while dragging and jumps only on release', async () => {
    const scrub = new DateScrub(() => feed());
    scrub.begin(0.3);
    expect(scrub.preview?.index).toBe(1);
    expect(scrollTo).toHaveBeenLastCalledWith('c');
    scrub.drag(0.8);
    expect(scrub.month(scrub.preview)?.month).toBe('2024-01');
    expect(searchFirst).not.toHaveBeenCalled();
    searchFirst.mockResolvedValue('z');
    const source = feed();
    const released = new DateScrub(() => source);
    released.begin(0.8);
    released.end(false);
    await vi.waitFor(() => expect(scrollTo).toHaveBeenLastCalledWith('z'));
    expect(released.preview).toBeNull();
    expect(source.jumpTo).toHaveBeenCalledWith('z', { failure: 'Jump to date' });
    expect(searchFirst).toHaveBeenCalledWith(
      { favorite: true, takenBefore: '2024-01-31T23:59:59.999Z' },
      { field: 'taken', direction: 'desc' }
    );
  });

  it('jumps when a month is cut off at the near end of the window', async () => {
    searchFirst.mockResolvedValue('a');
    const source = feed(true);
    const scrub = new DateScrub(() => source);
    expect(await scrub.go(0)).toBe(true);
    expect(source.jumpTo).toHaveBeenCalledWith('a', { failure: 'Jump to date' });
  });

  it('drops a slow jump overtaken by a newer one', async () => {
    let answer = (_value: unknown): void => undefined;
    searchFirst.mockImplementationOnce(() => new Promise((resolve) => (answer = resolve)));
    const source = feed();
    const scrub = new DateScrub(() => source);
    const slow = scrub.go(2);
    expect(await scrub.go(1)).toBe(true);
    answer('z');
    expect(await slow).toBe(false);
    expect(source.jumpTo).not.toHaveBeenCalled();
  });

  it('steps months and years from the keyboard', async () => {
    searchFirst.mockResolvedValue(null);
    const scrub = new DateScrub(() => feed());
    gridViewport.setTop('a');
    expect(scrub.key('ArrowDown')).toBe(true);
    expect(scrub.index).toBe(1);
    expect(scrollTo).toHaveBeenLastCalledWith('c');
    expect(scrub.key('PageDown')).toBe(true);
    expect(scrub.index).toBe(2);
    expect(scrub.key('Home')).toBe(true);
    expect(scrub.index).toBe(0);
    expect(scrub.key('x')).toBe(false);
    scrub.release();
    expect(scrub.index).toBe(0);
    expect(searchFirst).not.toHaveBeenCalled();
  });

  it('searches once for held keys after they settle', async () => {
    vi.useFakeTimers();
    try {
      searchFirst.mockResolvedValue('z');
      const source = feed();
      const scrub = new DateScrub(() => source);
      gridViewport.setTop('a');
      scrub.key('End');
      scrub.key('End');
      scrub.key('End');
      expect(searchFirst).not.toHaveBeenCalled();
      await vi.advanceTimersByTimeAsync(KEY_IDLE_MS);
      expect(searchFirst).toHaveBeenCalledOnce();
      expect(source.jumpTo).toHaveBeenCalledOnce();
      scrub.key('PageDown');
      scrub.keyUp();
      expect(searchFirst).toHaveBeenCalledTimes(2);
    } finally {
      vi.useRealTimers();
    }
  });

  it('reports a failed date search', async () => {
    searchFirst.mockRejectedValue(new Error('down'));
    const scrub = new DateScrub(() => feed());
    expect(await scrub.go(2)).toBe(false);
    expect(toasts.items.map((toast) => toast.message)).toEqual(['Jump to date: down']);
  });

  it('ticks only for touch drags', () => {
    const vibrate = vi.fn();
    vi.stubGlobal('navigator', { vibrate });
    try {
      const scrub = new DateScrub(() => feed());
      scrub.begin(0.1, 'mouse');
      scrub.drag(0.9, 'mouse');
      expect(vibrate).not.toHaveBeenCalled();
      scrub.begin(0.1, 'touch');
      scrub.drag(0.9, 'touch');
      expect(vibrate).toHaveBeenCalledOnce();
    } finally {
      vi.unstubAllGlobals();
    }
  });

  it('keeps the hover bubble apart from dragging', () => {
    const scrub = new DateScrub(() => feed());
    scrub.hover(0.9);
    expect(scrub.previewAt).toBeCloseTo(0.9);
    scrub.leave();
    expect(scrub.preview).toBeNull();
    scrub.begin(0.1);
    scrub.leave();
    expect(scrub.preview).not.toBeNull();
    scrub.cancel();
    expect(scrub.dragging).toBe(false);
  });
});
