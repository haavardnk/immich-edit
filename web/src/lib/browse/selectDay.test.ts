import { describe, expect, it, vi } from 'vitest';
import type { DayGroup } from './dayGroups';
import { dayAction, daySpan, loadDaySpan, pickDays, type DayPager } from './selectDay';

function groupsOf(days: string[]): DayGroup[] {
  const groups: DayGroup[] = [];
  days.forEach((key, index) => {
    const last = groups.at(-1);
    if (last && last.key === key) {
      last.end = index + 1;
      last.count += 1;
    } else groups.push({ key, start: index, end: index + 1, places: [], count: 1 });
  });
  return groups;
}

type Entry = { id: string; key: string };

function entries(days: string[], tag = ''): Entry[] {
  return days.map((key, index) => ({ id: `${key}${tag}${index}`, key }));
}

function indexIn(list: readonly Entry[]): (id: string) => number {
  return (id) => list.findIndex((entry) => entry.id === id);
}

class FakeFeed implements DayPager {
  items: Entry[];
  private before: Entry[][];
  private after: Entry[][];

  constructor(days: string[], before: string[][] = [], after: string[][] = []) {
    this.items = entries(days);
    this.before = before.map((page, index) => entries(page, `<${index}:`));
    this.after = after.map((page, index) => entries(page, `>${index}:`));
  }

  get hasPrevious(): boolean {
    return this.before.length > 0;
  }

  get hasMore(): boolean {
    return this.after.length > 0;
  }

  requestPrevious = vi.fn(async () => {
    const page = this.before.shift();
    if (!page) return false;
    this.items = [...page, ...this.items];
    return true;
  });

  requestMore = vi.fn(async () => {
    const page = this.after.shift();
    if (!page) return false;
    this.items = [...this.items, ...page];
    return true;
  });

  pager = this;
  groups = (): DayGroup[] => groupsOf(this.items.map((entry) => entry.key));
  indexOf = (id: string): number => indexIn(this.items)(id);
}

describe('daySpan', () => {
  const days = ['a', 'a', 'b', 'c', 'c', 'c'];
  const groups = groupsOf(days);
  const indexOf = indexIn(entries(days));

  it('covers one day', () => {
    expect(daySpan(groups, indexOf, 'c3', 'c5')).toEqual({ first: 2, last: 2, start: 3, end: 6 });
  });

  it('covers every day between two photos in either order', () => {
    expect(daySpan(groups, indexOf, 'c3', 'a0')).toEqual({ first: 0, last: 2, start: 0, end: 6 });
  });

  it('is null for an unknown photo', () => {
    expect(daySpan(groups, indexOf, 'a0', 'z9')).toBeNull();
  });

  it('tells apart two runs of the same day', () => {
    const split = ['a', 'b', 'a'];
    const span = daySpan(groupsOf(split), indexIn(entries(split)), 'a2', 'a2');
    expect(span).toEqual({ first: 2, last: 2, start: 2, end: 3 });
  });
});

describe('dayAction', () => {
  it('deselects only when every photo of the day is selected', () => {
    const selected = new Set(['1', '2']);
    expect(dayAction(['1', '2'], (id) => selected.has(id))).toBe('deselect');
    expect(dayAction(['1', '3'], (id) => selected.has(id))).toBe('select');
    expect(dayAction([], () => true)).toBe('select');
  });
});

describe('loadDaySpan', () => {
  it('loads before the near end while the day touches it', async () => {
    const feed = new FakeFeed(['b', 'c'], [['a', 'b', 'b']]);
    const span = await loadDaySpan(feed, 'b0', 'b0');
    expect(feed.requestPrevious).toHaveBeenCalledTimes(1);
    expect(span).toEqual({ first: 1, last: 1, start: 1, end: 4 });
  });

  it('keeps loading past the far end until the day ends', async () => {
    const feed = new FakeFeed(['a', 'b'], [], [['b', 'b'], ['b', 'c'], ['d']]);
    const span = await loadDaySpan(feed, 'b1', 'b1');
    expect(feed.requestMore).toHaveBeenCalledTimes(2);
    expect(span).toEqual({ first: 1, last: 1, start: 1, end: 5 });
  });

  it('does not load when the day sits inside the loaded range', async () => {
    const feed = new FakeFeed(['a', 'b', 'c'], [['z']], [['d']]);
    await loadDaySpan(feed, 'b1', 'b1');
    expect(feed.requestPrevious).not.toHaveBeenCalled();
    expect(feed.requestMore).not.toHaveBeenCalled();
  });

  it('stops when a load adds nothing', async () => {
    const feed = new FakeFeed(['a'], [[]]);
    expect(await loadDaySpan(feed, 'a0', 'a0')).toEqual({
      first: 0,
      last: 0,
      start: 0,
      end: 1
    });
  });

  it('gives up when a load fails', async () => {
    const feed = new FakeFeed(['a'], [['z']]);
    feed.requestPrevious.mockResolvedValueOnce(false);
    expect(await loadDaySpan(feed, 'a0', 'a0')).toBeNull();
  });
});

describe('pickDays', () => {
  function picker(feed: FakeFeed, selected: Set<string>) {
    return {
      pager: feed,
      groups: feed.groups,
      indexOf: feed.indexOf,
      ids: (start: number, end: number) => feed.items.slice(start, end).map((entry) => entry.id),
      isSelected: (id: string) => selected.has(id),
      apply: (ids: string[], on: boolean) => {
        for (const id of ids) {
          if (on) selected.add(id);
          else selected.delete(id);
        }
      }
    };
  }

  it('toggles a whole day', async () => {
    const feed = new FakeFeed(['a', 'b', 'b', 'c']);
    const selected = new Set<string>();
    expect(await pickDays(picker(feed, selected), 'b1', 'b1')).toBe(true);
    expect([...selected]).toEqual(['b1', 'b2']);
    expect(await pickDays(picker(feed, selected), 'b1', 'b1')).toBe(false);
    expect(selected.size).toBe(0);
  });

  it('selects every day of an extended range even when some are selected', async () => {
    const feed = new FakeFeed(['a', 'b', 'c']);
    const selected = new Set(['a0', 'b1', 'c2']);
    expect(await pickDays(picker(feed, selected), 'a0', 'c2')).toBe(true);
    expect(selected.size).toBe(3);
  });

  it('selects only the run of a day whose header was picked', async () => {
    const feed = new FakeFeed(['a', 'a', 'b', 'a']);
    const selected = new Set<string>();
    expect(await pickDays(picker(feed, selected), 'a3', 'a3')).toBe(true);
    expect([...selected]).toEqual(['a3']);
  });

  it('extends across both runs of a day that comes back', async () => {
    const feed = new FakeFeed(['a', 'b', 'a']);
    const selected = new Set(['a0', 'b1', 'a2']);
    expect(await pickDays(picker(feed, selected), 'a0', 'a2')).toBe(true);
    expect(selected.size).toBe(3);
  });

  it('toggles off when the range stays inside one selected day', async () => {
    const feed = new FakeFeed(['a', 'b', 'b']);
    const selected = new Set(['b1', 'b2']);
    expect(await pickDays(picker(feed, selected), 'b1', 'b2')).toBe(false);
    expect(selected.size).toBe(0);
  });

  it('finds the picked day again after earlier photos join it', async () => {
    const feed = new FakeFeed(['b', 'c'], [['a', 'b']]);
    const selected = new Set<string>();
    expect(await pickDays(picker(feed, selected), 'b0', 'b0')).toBe(true);
    expect([...selected]).toEqual(['b<0:1', 'b0']);
  });
});
