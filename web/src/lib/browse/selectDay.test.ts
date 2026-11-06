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
    } else groups.push({ key, start: index, end: index + 1, place: null, count: 1 });
  });
  return groups;
}

class FakeFeed implements DayPager {
  constructor(
    public days: string[],
    private before: string[][] = [],
    private after: string[][] = []
  ) {}

  get hasPrevious(): boolean {
    return this.before.length > 0;
  }

  get hasMore(): boolean {
    return this.after.length > 0;
  }

  requestPrevious = vi.fn(async () => {
    const page = this.before.shift();
    if (!page) return false;
    this.days = [...page, ...this.days];
    return true;
  });

  requestMore = vi.fn(async () => {
    const page = this.after.shift();
    if (!page) return false;
    this.days = [...this.days, ...page];
    return true;
  });

  groups = (): DayGroup[] => groupsOf(this.days);
}

describe('daySpan', () => {
  const groups = groupsOf(['a', 'a', 'b', 'c', 'c', 'c']);

  it('covers one day', () => {
    expect(daySpan(groups, 'c', 'c')).toEqual({ first: 2, last: 2, start: 3, end: 6 });
  });

  it('covers every day between two keys in either order', () => {
    expect(daySpan(groups, 'c', 'a')).toEqual({ first: 0, last: 2, start: 0, end: 6 });
  });

  it('is null for an unknown day', () => {
    expect(daySpan(groups, 'a', 'z')).toBeNull();
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
    const span = await loadDaySpan(feed, feed.groups, 'b', 'b');
    expect(feed.requestPrevious).toHaveBeenCalledTimes(1);
    expect(span).toEqual({ first: 1, last: 1, start: 1, end: 4 });
  });

  it('keeps loading past the far end until the day ends', async () => {
    const feed = new FakeFeed(['a', 'b'], [], [['b', 'b'], ['b', 'c'], ['d']]);
    const span = await loadDaySpan(feed, feed.groups, 'b', 'b');
    expect(feed.requestMore).toHaveBeenCalledTimes(2);
    expect(span).toEqual({ first: 1, last: 1, start: 1, end: 5 });
  });

  it('does not load when the day sits inside the loaded range', async () => {
    const feed = new FakeFeed(['a', 'b', 'c'], [['z']], [['d']]);
    await loadDaySpan(feed, feed.groups, 'b', 'b');
    expect(feed.requestPrevious).not.toHaveBeenCalled();
    expect(feed.requestMore).not.toHaveBeenCalled();
  });

  it('stops when a load adds nothing', async () => {
    const feed = new FakeFeed(['a'], [[]]);
    expect(await loadDaySpan(feed, feed.groups, 'a', 'a')).toEqual({
      first: 0,
      last: 0,
      start: 0,
      end: 1
    });
  });

  it('gives up when a load fails', async () => {
    const feed = new FakeFeed(['a'], [['z']]);
    feed.requestPrevious.mockResolvedValueOnce(false);
    expect(await loadDaySpan(feed, feed.groups, 'a', 'a')).toBeNull();
  });
});

describe('pickDays', () => {
  function picker(feed: FakeFeed, selected: Set<string>) {
    return {
      pager: feed,
      groups: feed.groups,
      ids: (start: number, end: number) =>
        Array.from(
          { length: end - start },
          (_, offset) => `${feed.days[start + offset]}${start + offset}`
        ),
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
    expect(await pickDays(picker(feed, selected), 'b', 'b', false)).toBe(true);
    expect([...selected]).toEqual(['b1', 'b2']);
    expect(await pickDays(picker(feed, selected), 'b', 'b', false)).toBe(false);
    expect(selected.size).toBe(0);
  });

  it('selects every day of an extended range even when some are selected', async () => {
    const feed = new FakeFeed(['a', 'b', 'c']);
    const selected = new Set(['a0', 'b1', 'c2']);
    expect(await pickDays(picker(feed, selected), 'a', 'c', true)).toBe(true);
    expect(selected.size).toBe(3);
  });
});
