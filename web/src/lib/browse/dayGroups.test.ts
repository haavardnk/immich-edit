import { describe, expect, it } from 'vitest';
import { dayKey, daySubtitle, dayTitle, groupDays } from './dayGroups';
import type { ExifInfo } from '#lib/types/asset.ts';

function shot(localDateTime: string | null, city: string | null = null, fileCreatedAt = '') {
  return {
    localDateTime,
    fileCreatedAt: fileCreatedAt || localDateTime,
    exifInfo: { city } as ExifInfo
  };
}

describe('dayKey', () => {
  it('reads the wall-clock day from localDateTime', () => {
    expect(dayKey(shot('2026-10-31T23:30:00.000Z'))).toBe('2026-10-31');
  });

  it('falls back to fileCreatedAt in the browser zone', () => {
    const created = new Date(2025, 6, 4, 12).toISOString();
    expect(dayKey({ localDateTime: null, fileCreatedAt: created, exifInfo: null })).toBe(
      '2025-07-04'
    );
  });

  it('uses the EXIF zone of dateTimeOriginal when there is no local time', () => {
    const asset = {
      localDateTime: null,
      fileCreatedAt: null,
      exifInfo: { dateTimeOriginal: '2026-01-01T02:00:00Z', timeZone: 'America/New_York' }
    } as unknown as Parameters<typeof dayKey>[0];
    expect(dayKey(asset)).toBe('2025-12-31');
  });

  it('gives an empty key when no time is known', () => {
    expect(dayKey({ localDateTime: null, fileCreatedAt: null, exifInfo: null })).toBe('');
  });
});

describe('groupDays', () => {
  it('splits consecutive runs by day with the commonest city and the count', () => {
    const groups = groupDays([
      shot('2026-10-31T18:00:00Z', 'Bergen'),
      shot('2026-10-31T12:00:00Z', 'Oslo'),
      shot('2026-10-31T09:00:00Z', 'Bergen'),
      shot('2026-10-30T20:00:00Z', null),
      shot('2026-10-30T10:00:00Z', null)
    ]);
    expect(groups).toEqual([
      { key: '2026-10-31', start: 0, end: 3, place: 'Bergen', count: 3 },
      { key: '2026-10-30', start: 3, end: 5, place: null, count: 2 }
    ]);
  });

  it('counts masters only and keeps copies inside their day', () => {
    const master = shot('2026-10-31T18:00:00Z', 'Bergen');
    const groups = groupDays([
      master,
      { ...master, copyOf: 'm' },
      shot('2026-10-31T09:00:00Z', null)
    ]);
    expect(groups).toEqual([{ key: '2026-10-31', start: 0, end: 3, place: 'Bergen', count: 2 }]);
  });

  it('keeps a day that comes back later as its own run', () => {
    const groups = groupDays([
      shot('2026-10-31T18:00:00Z'),
      shot('2026-10-30T18:00:00Z'),
      shot('2026-10-31T01:00:00Z')
    ]);
    expect(groups.map((group) => [group.key, group.start, group.end])).toEqual([
      ['2026-10-31', 0, 1],
      ['2026-10-30', 1, 2],
      ['2026-10-31', 2, 3]
    ]);
  });

  it('returns no groups for an empty list', () => {
    expect(groupDays([])).toEqual([]);
  });
});

describe('dayTitle', () => {
  const today = new Date(2026, 10, 5);

  it('names the weekday, day and month in the current year', () => {
    expect(dayTitle('2026-10-31', today)).toBe('Saturday 31 October');
  });

  it('appends the year for other years', () => {
    expect(dayTitle('2025-07-04', today)).toBe('Friday 4 July 2025');
  });

  it('labels an unknown day', () => {
    expect(dayTitle('', today)).toBe('Unknown date');
  });
});

describe('daySubtitle', () => {
  it('joins the place and count', () => {
    expect(daySubtitle({ place: 'Bergen', count: 24 })).toBe('Bergen · 24');
  });

  it('shows only the count without a place', () => {
    expect(daySubtitle({ place: null, count: 3 })).toBe('3');
  });
});
