import type { PhotoMatch } from '#lib/geotag/match.ts';
import { planWrites } from '#lib/metadata/captureTimes.ts';
import { cameraTime, wallToInstant } from '#lib/metadata/photoTime.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { describe, expect, it } from 'vitest';
import { trackCaptureChanges } from './trackCapture';

const SUMMER = Date.UTC(2026, 6, 15, 8);
const OSLO = 'Europe/Oslo';
const TANZANIA = 'Africa/Dar_es_Salaam';

function asset(id: string, wall: number, timeZone: string | null): AssetSummary {
  const stored = timeZone ? new Date(wall - 2 * 3_600_000).toISOString() : null;
  return {
    id,
    originalFileName: `${id}.jpg`,
    localDateTime: new Date(wall).toISOString(),
    exifInfo: { make: 'Sony', model: 'ILCE-7M3', dateTimeOriginal: stored, timeZone }
  } as AssetSummary;
}

function row(photo: AssetSummary, shiftMs: number, zone: string | null, track = OSLO) {
  const base = zone
    ? wallToInstant(Date.parse(photo.localDateTime ?? ''), zone)
    : (cameraTime(photo, track)?.instant ?? null);
  const time = base === null ? null : base + shiftMs;
  const match: PhotoMatch = { id: photo.id, time, outcome: { kind: 'untimed' } };
  return { asset: photo, match, shiftMs, zone };
}

describe('track capture changes', () => {
  it('writes a 22 second shift to the second', () => {
    const photo = asset('a', SUMMER, OSLO);
    const plan = planWrites(trackCaptureChanges([row(photo, 22_000, null)], OSLO));
    expect(plan.write).toEqual([{ id: 'a', at: '2026-07-15T06:00:22.000Z', timeZone: OSLO }]);
    expect(plan.undo).toEqual([{ id: 'a', at: '2026-07-15T06:00:00.000Z', timeZone: OSLO }]);
  });

  it('saves a camera set to Tanzania time in that time zone', () => {
    const changes = trackCaptureChanges([row(asset('b', SUMMER, null), 0, TANZANIA)], OSLO);
    expect(planWrites(changes).write).toEqual([
      { id: 'b', at: '2026-07-15T05:00:00.000Z', timeZone: TANZANIA }
    ]);
  });

  it('moves a photo shot in the wrong time zone to the picked one', () => {
    const changes = trackCaptureChanges([row(asset('c', SUMMER, OSLO), 0, TANZANIA)], OSLO);
    expect(planWrites(changes).write).toEqual([{ id: 'c', minutes: -60, timeZone: TANZANIA }]);
  });

  it('writes nothing when no camera changed', () => {
    const rows = [
      row(asset('d', SUMMER, OSLO), 0, null),
      row(asset('e', SUMMER, null), 0, null),
      row(asset('f', SUMMER, OSLO), 0, OSLO)
    ];
    expect(trackCaptureChanges(rows, OSLO)).toEqual([]);
  });

  it('leaves untimed photos alone', () => {
    const photo = asset('g', SUMMER, OSLO);
    const untimed = {
      asset: photo,
      shiftMs: 60_000,
      zone: null,
      match: { id: 'g', time: null, outcome: { kind: 'untimed' as const } }
    };
    expect(trackCaptureChanges([untimed], OSLO)).toEqual([]);
  });
});
