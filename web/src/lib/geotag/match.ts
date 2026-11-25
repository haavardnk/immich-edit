import type { Located, LocateOptions, TrackLog } from '#lib/geotag/trackLog.ts';

export interface TimedPhoto {
  id: string;
  base: number | null;
  shiftMs: number;
  located: boolean;
}

export type PhotoOutcome = Located | { kind: 'kept' } | { kind: 'untimed' };

export interface PhotoMatch {
  id: string;
  time: number | null;
  outcome: PhotoOutcome;
}

export interface MatchOptions extends LocateOptions {
  replace: boolean;
}

export function matchPhotos(
  photos: readonly TimedPhoto[],
  log: TrackLog,
  options: MatchOptions
): PhotoMatch[] {
  return photos.map((photo) => {
    if (photo.base === null) return { id: photo.id, time: null, outcome: { kind: 'untimed' } };
    const time = photo.base + photo.shiftMs;
    if (photo.located && !options.replace) return { id: photo.id, time, outcome: { kind: 'kept' } };
    return { id: photo.id, time, outcome: log.locate(time, options) };
  });
}

export interface MatchCounts {
  placed: number;
  outside: number;
  kept: number;
  untimed: number;
}

export function countMatches(matches: readonly PhotoMatch[]): MatchCounts {
  const counts: MatchCounts = { placed: 0, outside: 0, kept: 0, untimed: 0 };
  for (const { outcome } of matches) counts[outcome.kind] += 1;
  return counts;
}
