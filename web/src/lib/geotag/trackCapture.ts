import type { PhotoMatch } from '#lib/geotag/match.ts';
import { clockPhoto, type ClockChange } from '#lib/metadata/captureTimes.ts';
import type { AssetSummary } from '#lib/types/album.ts';

export interface CaptureRow {
  asset: AssetSummary;
  match: PhotoMatch;
  shiftMs: number;
  zone: string | null;
}

export function trackCaptureChanges(rows: readonly CaptureRow[], trackZone: string): ClockChange[] {
  return rows.flatMap(({ asset, match, shiftMs, zone }) => {
    if (match.time === null) return [];
    const photo = clockPhoto(asset);
    if (!photo) return [];
    if (shiftMs === 0 && (zone === null || zone === photo.zone)) return [];
    return [{ photo, target: { instant: match.time, zone: zone ?? photo.zone ?? trackZone } }];
  });
}
