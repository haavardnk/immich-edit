import type { PhotoMatch } from '#lib/geotag/match.ts';
import { planWrites } from '#lib/metadata/captureTimes.ts';
import { cameraTime } from '#lib/metadata/photoTime.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { describe, expect, it } from 'vitest';
import { trackCaptureChanges, type CaptureZones } from './trackCapture';

const SUMMER = Date.UTC(2026, 6, 15, 8);
const OSLO: CaptureZones = { clock: 'Europe/Oslo', track: 'Europe/Oslo' };

function asset(id: string, wall: number, timeZone: string | null): AssetSummary {
  const stored = timeZone ? new Date(wall - 2 * 3_600_000).toISOString() : null;
  return {
    id,
    originalFileName: `${id}.jpg`,
    localDateTime: new Date(wall).toISOString(),
    exifInfo: { make: 'Sony', model: 'ILCE-7M3', dateTimeOriginal: stored, timeZone }
  } as AssetSummary;
}

function row(photo: AssetSummary, shiftMs: number, zones: CaptureZones) {
  const base = cameraTime(photo, zones.clock)?.instant ?? null;
  const time = base === null ? null : base + shiftMs;
  const match: PhotoMatch = { id: photo.id, time, outcome: { kind: 'untimed' } };
  return { asset: photo, match, shiftMs };
}

describe('track capture changes', () => {
  it('writes a 22 second shift to the second', () => {
    const photo = asset('a', SUMMER, 'Europe/Oslo');
    const plan = planWrites(trackCaptureChanges([row(photo, 22_000, OSLO)], OSLO));
    expect(plan.write).toEqual([
      { id: 'a', at: '2026-07-15T06:00:22.000Z', timeZone: 'Europe/Oslo' }
    ]);
    expect(plan.undo).toEqual([
      { id: 'a', at: '2026-07-15T06:00:00.000Z', timeZone: 'Europe/Oslo' }
    ]);
  });

  it('saves an unzoned Oslo camera in Tanzania as Tanzania time', () => {
    const zones = { clock: 'Europe/Oslo', track: 'Africa/Dar_es_Salaam' };
    const plan = planWrites(trackCaptureChanges([row(asset('b', SUMMER, null), 0, zones)], zones));
    expect(plan.write).toEqual([
      { id: 'b', at: '2026-07-15T06:00:00.000Z', timeZone: 'Africa/Dar_es_Salaam' }
    ]);
  });

  it('writes nothing when no camera clock moved', () => {
    const rows = [
      row(asset('c', SUMMER, 'Europe/Oslo'), 0, OSLO),
      row(asset('d', SUMMER, null), 0, OSLO)
    ];
    expect(trackCaptureChanges(rows, OSLO)).toEqual([]);
  });

  it('leaves untimed photos alone', () => {
    const photo = asset('e', SUMMER, 'Europe/Oslo');
    const untimed = {
      asset: photo,
      shiftMs: 60_000,
      match: { id: 'e', time: null, outcome: { kind: 'untimed' as const } }
    };
    expect(trackCaptureChanges([untimed], OSLO)).toEqual([]);
  });
});
