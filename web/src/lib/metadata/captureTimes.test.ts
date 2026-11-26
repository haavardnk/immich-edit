import {
  baseMoment,
  clockPhoto,
  correctionTo,
  restoreTarget,
  planWrites,
  previewRows,
  setTo,
  shifted,
  wallOf,
  writeFor,
  type ClockPhoto,
  type ClockTarget
} from '#lib/metadata/captureTimes.ts';
import { wallToInstant } from '#lib/metadata/photoTime.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { describe, expect, it } from 'vitest';

const MIN = 60_000;
const at = (iso: string): number => Date.parse(iso);

function asset(
  id: string,
  exif: Partial<NonNullable<AssetSummary['exifInfo']>>,
  local: string
): AssetSummary {
  return {
    id,
    originalFileName: `${id}.ARW`,
    type: 'IMAGE',
    localDateTime: local,
    fileCreatedAt: local,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: { make: 'Sony', model: 'ILCE-7M3', ...exif } as AssetSummary['exifInfo'],
    tags: []
  };
}

const zoneless = clockPhoto(
  asset(
    'a',
    { dateTimeOriginal: '2026-10-31T13:03:12.345+00:00', timeZone: null },
    '2026-10-31T13:03:12.345Z'
  )
) as ClockPhoto;
const zoned = clockPhoto(
  asset(
    'b',
    { dateTimeOriginal: '2026-10-31T12:03:12.345+00:00', timeZone: 'UTC+1' },
    '2026-10-31T13:03:12.345Z'
  )
) as ClockPhoto;
const asRecorded = { takenIn: null, clock: null, fallback: 'Europe/Oslo' };

describe('capture times', () => {
  it('reads the shown time, the stored time and the zone', () => {
    expect(zoneless).toMatchObject({ wall: at('2026-10-31T13:03:12.345Z'), zone: null });
    expect(zoned).toMatchObject({ stored: at('2026-10-31T12:03:12.345Z'), zone: 'UTC+1' });
    const odd = clockPhoto(
      asset(
        'c',
        { dateTimeOriginal: '2026-10-31T12:00:00Z', timeZone: 'Mars/Base' },
        '2026-10-31T12:00:00Z'
      )
    );
    expect(odd?.zone).toBeNull();
  });

  it('shifts a zoned photo by its stored moment and keeps its zone', () => {
    const target = shifted(zoned, 62 * MIN + 37_000, asRecorded) as ClockTarget;
    expect(target).toEqual({ instant: at('2026-10-31T13:05:49.345Z'), zone: 'UTC+1' });
    expect(wallOf(target)).toBe(at('2026-10-31T14:05:49.345Z'));
  });

  it('leaves a photo alone when neither its time nor its zone changes', () => {
    expect(shifted(zoneless, 0, asRecorded)).toBeNull();
  });

  it('gives a shifted zoneless photo the fallback zone', () => {
    const target = shifted(zoneless, 2 * MIN, asRecorded) as ClockTarget;
    expect(target.zone).toBe('Europe/Oslo');
    expect(wallOf(target)).toBe(zoneless.wall + 2 * MIN);
  });

  it('reads the shown time in the camera clock zone and shows it where it was taken', () => {
    const target = shifted(zoneless, 0, {
      takenIn: 'Asia/Bangkok',
      clock: 'Europe/Oslo',
      fallback: 'UTC'
    });
    expect(new Date(wallOf(target!)).toISOString()).toBe('2026-10-31T19:03:12.345Z');
    const same = shifted(zoned, 0, {
      takenIn: 'Asia/Bangkok',
      clock: null,
      fallback: 'UTC'
    }) as ClockTarget;
    expect(wallOf(same!)).toBe(zoned.wall);
  });

  it('writes whole minutes as a shift and anything else as a set time', () => {
    const minutes = writeFor('a', at('2026-10-31T12:00:00Z'), 'UTC+1', {
      instant: at('2026-10-31T13:02:00Z'),
      zone: 'UTC+1'
    });
    expect(minutes).toEqual({ id: 'a', minutes: 62 });
    const zone = writeFor('a', at('2026-10-31T12:00:00Z'), null, {
      instant: at('2026-10-31T12:00:00Z'),
      zone: 'Europe/Oslo'
    });
    expect(zone).toEqual({ id: 'a', minutes: 0, timeZone: 'Europe/Oslo' });
    const seconds = writeFor('a', at('2026-10-31T12:00:00Z'), 'UTC+1', {
      instant: at('2026-10-31T12:00:37.5Z'),
      zone: 'UTC+1'
    });
    expect(seconds).toEqual({ id: 'a', at: '2026-10-31T12:00:37.500Z', timeZone: 'UTC+1' });
    const nothing = writeFor('a', at('2026-10-31T12:00:00Z'), 'UTC+1', {
      instant: at('2026-10-31T12:00:00Z'),
      zone: 'UTC+1'
    });
    expect(nothing).toBeNull();
    const undated = writeFor('a', null, null, {
      instant: at('1987-06-01T10:00:00Z'),
      zone: 'Europe/Oslo'
    });
    expect(undated).toEqual({ id: 'a', at: '1987-06-01T10:00:00.000Z', timeZone: 'Europe/Oslo' });
  });

  it('plans an undo that brings back each photo as it was', () => {
    const plan = planWrites([
      { photo: zoned, target: shifted(zoned, 37_000, asRecorded) as ClockTarget },
      { photo: zoneless, target: shifted(zoneless, 2 * MIN, asRecorded) as ClockTarget }
    ]);
    expect(plan.write).toHaveLength(2);
    expect(plan.undo[0]).toEqual({ id: 'b', at: '2026-10-31T12:03:12.345Z', timeZone: 'UTC+1' });
    const back = plan.undo[1] as { id: string; minutes: number };
    expect(back.minutes).toBe(-2);
  });

  it('orders the preview by the new time and marks photos that pass another', () => {
    const other = clockPhoto(
      asset(
        'z',
        { dateTimeOriginal: '2026-10-31T13:04:00+00:00', timeZone: 'UTC+1', model: 'iPhone' },
        '2026-10-31T14:04:00Z'
      )
    ) as ClockPhoto;
    const targets = new Map<string, ClockTarget>([
      ['b', shifted(zoned, 62 * MIN, asRecorded) as ClockTarget]
    ]);
    const rows = previewRows([zoned, other], targets);
    expect(rows.map((row) => row.photo.id)).toEqual(['z', 'b']);
    expect(rows.map((row) => [row.changed, row.reordered])).toEqual([
      [false, false],
      [true, true]
    ]);
  });

  it('sets a wall time in a zone', () => {
    expect(wallOf(setTo(at('1987-06-01T12:00:00Z'), 'Europe/Oslo'))).toBe(
      at('1987-06-01T12:00:00Z')
    );
  });

  it('corrects a zoned photo across a DST change to exactly the time typed', () => {
    const oslo = clockPhoto(
      asset(
        'o',
        { dateTimeOriginal: '2026-03-28T22:00:00.000+00:00', timeZone: 'Europe/Oslo' },
        '2026-03-28T23:00:00.000Z'
      )
    ) as ClockPhoto;
    const moment = wallToInstant(Date.parse('2026-03-29T03:30:00.000Z'), 'Europe/Oslo');
    const correction = correctionTo(oslo, moment, asRecorded);
    const target = shifted(oslo, correction, asRecorded) as ClockTarget;
    expect(wallOf(target)).toBe(Date.parse('2026-03-29T03:30:00.000Z'));
    expect(baseMoment(oslo, asRecorded)).toBe(oslo.stored);
  });

  it('restores a zoned photo to its stored moment in the repeated fall-back hour', () => {
    const early = clockPhoto(
      asset(
        'e',
        { dateTimeOriginal: '2026-10-25T00:30:00.000+00:00', timeZone: 'Europe/Oslo' },
        '2026-10-25T02:30:00.000Z'
      )
    ) as ClockPhoto;
    const after = shifted(early, MIN, asRecorded) as ClockTarget;
    expect(restoreTarget(early, after).instant).toBe(Date.parse('2026-10-25T00:30:00.000Z'));
  });

  it('matches a zoneless photo to a moment in the fallback zone', () => {
    const moment = Date.parse('2026-10-31T13:04:12.345Z');
    const correction = correctionTo(zoneless, moment, asRecorded);
    expect((shifted(zoneless, correction, asRecorded) as ClockTarget).instant).toBe(moment);
  });
});
