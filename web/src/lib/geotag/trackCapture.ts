import type { PhotoMatch } from '#lib/geotag/match.ts';
import { clockPhoto, type ClockChange } from '#lib/metadata/captureTimes.ts';
import type { AssetSummary } from '#lib/types/album.ts';

export interface CaptureRow {
  asset: AssetSummary;
  match: PhotoMatch;
  shiftMs: number;
}

export interface CaptureZones {
  clock: string;
  track: string;
}

export function trackCaptureChanges(
  rows: readonly CaptureRow[],
  zones: CaptureZones
): ClockChange[] {
  const rezoned = zones.clock !== zones.track;
  return rows.flatMap(({ asset, match, shiftMs }) => {
    if (match.time === null) return [];
    const photo = clockPhoto(asset);
    if (!photo) return [];
    if (shiftMs === 0 && (photo.zone !== null || !rezoned)) return [];
    return [{ photo, target: { instant: match.time, zone: photo.zone ?? zones.track } }];
  });
}
