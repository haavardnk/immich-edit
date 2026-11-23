import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { gridHost } from '#lib/commands/grid/gridHost.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { browseControls } from '#lib/stores/browseControls.svelte.ts';
import { monthCounts } from '#lib/stores/monthCounts.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { DateScrub, KEY_IDLE_MS, type ScrubFeed } from './dateScrub.svelte';
import { gridViewport } from './gridViewport.svelte';
import { dayKey } from './dayGroups';
import { localDays } from './scrubDays';

const searchLocate = vi.hoisted(() => vi.fn());
const searchCounts = vi.hoisted(() => vi.fn());

vi.mock('#lib/api/search.ts', () => ({ searchLocate, searchCounts, searchMonths: vi.fn() }));

function photo(id: string, fileCreatedAt: string): AssetSummary {
  return { id, fileCreatedAt, exifInfo: null } as AssetSummary;
}

const scrollTo = vi.fn();
let detach = (): void => {};

type FakeFeed = ScrubFeed & { hasPrevious: boolean; jumpTo: ReturnType<typeof vi.fn> };

function feed(hasPrevious = false): FakeFeed {
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
    dragging: () => false,
    cancelDrag: () => {},
    holdAnchor: () => () => {}
  });
});

afterEach(() => {
  detach();
  scrollTo.mockReset();
  searchLocate.mockReset();
  searchCounts.mockReset();
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
    expect(searchLocate).not.toHaveBeenCalled();
    expect(source.jumpTo).not.toHaveBeenCalled();
  });

  it('locates an unloaded photo deep in a month and keeps seeking around it', async () => {
    monthCounts.months = [{ month: '2026-10', count: 9000 }];
    browsing.set([photo('a', '2026-10-31T00:00:00Z'), photo('b', '2026-10-30T00:00:00Z')]);
    const source = feed();
    source.jumpTo.mockImplementation(async () => {
      source.hasPrevious = true;
      browsing.set(['w', 'x', 'y', 'z'].map((name, at) => photo(name, `2026-10-15T00:00:0${at}Z`)));
      return true;
    });
    searchLocate.mockResolvedValue('x');
    const scrub = new DateScrub(() => source);
    scrub.begin(0.5);
    expect(scrollTo).toHaveBeenLastCalledWith('b');
    scrub.end(false);
    await vi.waitFor(() => expect(scrollTo).toHaveBeenLastCalledWith('x'));
    expect(searchLocate).toHaveBeenCalledWith({ favorite: true }, 'desc', '2026-10', 4500);
    expect(source.jumpTo).toHaveBeenCalledWith('x', { failure: 'Jump to date' });
    gridViewport.setTop('y');
    expect(scrub.visible).toEqual({ index: 0, within: 4501 / 9000 });
    scrub.begin(4502.5 / 9000);
    expect(scrollTo).toHaveBeenLastCalledWith('z');
    scrub.end(false);
    await Promise.resolve();
    expect(searchLocate).toHaveBeenCalledOnce();
  });

  it('names the day under the pointer from day counts or the loaded photo', async () => {
    const january = localDays('2024-01');
    searchCounts.mockImplementation(async (_filter: unknown, edges: string[]) =>
      edges.slice(1).map((_, at) => (at === 9 ? 3 : at === 20 ? 1 : 0))
    );
    const scrub = new DateScrub(() => feed());
    scrub.hover(0.6);
    expect(scrub.month(scrub.preview)?.month).toBe('2024-01');
    expect(scrub.previewDay).toBeNull();
    await vi.waitFor(() => expect(scrub.previewDay).toEqual({ key: january?.keys[20], count: 1 }));
    expect(searchCounts).toHaveBeenCalledWith({ favorite: true }, january?.edges);
    scrub.hover(0.99);
    expect(scrub.previewDay).toEqual({ key: january?.keys[9], count: 3 });
    scrub.hover(0.05);
    const first = browsing.visible[0];
    await vi.waitFor(() => expect(searchCounts).toHaveBeenCalledTimes(2));
    expect(scrub.previewDay?.key).toBe(first ? dayKey(first) : null);
    scrub.hover(0.62);
    expect(searchCounts).toHaveBeenCalledTimes(2);
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
    expect(searchLocate).not.toHaveBeenCalled();
    searchLocate.mockResolvedValue('z');
    const source = feed();
    const released = new DateScrub(() => source);
    released.begin(0.8);
    released.end(false);
    await vi.waitFor(() => expect(scrollTo).toHaveBeenLastCalledWith('z'));
    expect(released.preview).toBeNull();
    expect(source.jumpTo).toHaveBeenCalledWith('z', { failure: 'Jump to date' });
    expect(searchLocate).toHaveBeenCalledWith({ favorite: true }, 'desc', '2024-01', 2);
  });

  it('jumps when a month is cut off at the near end of the window', async () => {
    searchLocate.mockResolvedValue('a');
    const source = feed(true);
    const scrub = new DateScrub(() => source);
    expect(await scrub.go(0)).toBe(true);
    expect(source.jumpTo).toHaveBeenCalledWith('a', { failure: 'Jump to date' });
  });

  it('drops a slow jump when a new drag starts', async () => {
    let answer = (_value: unknown): void => undefined;
    searchLocate.mockImplementationOnce(() => new Promise((resolve) => (answer = resolve)));
    const source = feed();
    const scrub = new DateScrub(() => source);
    const slow = scrub.go(2);
    scrub.begin(0.1);
    answer('z');
    expect(await slow).toBe(false);
    expect(source.jumpTo).not.toHaveBeenCalled();
  });

  it('drops a slow jump overtaken by a newer one', async () => {
    let answer = (_value: unknown): void => undefined;
    searchLocate.mockImplementationOnce(() => new Promise((resolve) => (answer = resolve)));
    const source = feed();
    const scrub = new DateScrub(() => source);
    const slow = scrub.go(2);
    expect(await scrub.go(1)).toBe(true);
    answer('z');
    expect(await slow).toBe(false);
    expect(source.jumpTo).not.toHaveBeenCalled();
  });

  it('steps months and years from the keyboard', async () => {
    searchLocate.mockResolvedValue(null);
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
    expect(searchLocate).not.toHaveBeenCalled();
  });

  it('searches once for held keys after they settle', async () => {
    vi.useFakeTimers();
    try {
      searchLocate.mockResolvedValue('z');
      const source = feed();
      const scrub = new DateScrub(() => source);
      gridViewport.setTop('a');
      scrub.key('End');
      scrub.key('End');
      scrub.key('End');
      expect(searchLocate).not.toHaveBeenCalled();
      await vi.advanceTimersByTimeAsync(KEY_IDLE_MS);
      expect(searchLocate).toHaveBeenCalledOnce();
      expect(source.jumpTo).toHaveBeenCalledOnce();
      scrub.key('PageDown');
      scrub.keyUp();
      expect(searchLocate).toHaveBeenCalledTimes(2);
    } finally {
      vi.useRealTimers();
    }
  });

  it('reports a failed date search', async () => {
    searchLocate.mockRejectedValue(new Error('down'));
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
