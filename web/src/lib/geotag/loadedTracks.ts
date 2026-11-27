import { distanceText } from '#lib/geotag/spots.ts';
import type { TrackPoint } from '#lib/geotag/trackFile.ts';
import { TrackLog } from '#lib/geotag/trackLog.ts';
import { correctionText } from '#lib/metadata/clockText.ts';
import { countOf } from '#lib/utils/plural.ts';
import { DateTime } from 'luxon';

export interface LoadedTrack {
  id: string;
  fileName: string;
  name: string | null;
  segments: TrackPoint[][];
  points: number;
  start: number;
  end: number;
  metres: number;
  offsetMs: number;
}

export function loadedTrack(
  fileName: string,
  name: string | null,
  segments: TrackPoint[][],
  id: number
): LoadedTrack {
  const log = new TrackLog(segments);
  return {
    id: `track-${id}`,
    fileName,
    name,
    segments,
    points: log.size,
    start: log.start,
    end: log.end,
    metres: log.lengthMetres(),
    offsetMs: 0
  };
}

function shiftedSegments(track: LoadedTrack): TrackPoint[][] {
  if (track.offsetMs === 0) return track.segments;
  return track.segments.map((points) =>
    points.map((point) => ({ ...point, time: point.time + track.offsetMs }))
  );
}

export function trackLogOf(tracks: readonly LoadedTrack[]): TrackLog {
  return new TrackLog(tracks.flatMap(shiftedSegments));
}

export function trackName(track: LoadedTrack): string {
  return track.name ?? track.fileName;
}

function when(start: number, end: number, zone: string): string {
  const from = DateTime.fromMillis(start, { zone });
  const to = DateTime.fromMillis(end, { zone });
  if (from.hasSame(to, 'day')) {
    return `${from.toFormat('d LLL')} · ${from.toFormat('HH:mm')}–${to.toFormat('HH:mm')}`;
  }
  return `${from.toFormat('d LLL')} – ${to.toFormat('d LLL')}`;
}

export function trackShiftText(tracks: readonly LoadedTrack[]): string {
  const offsets = new Set(tracks.map((track) => track.offsetMs));
  const shifted = tracks.filter((track) => track.offsetMs !== 0).length;
  if (shifted === 0) return '';
  const [only] = offsets;
  if (offsets.size === 1 && only !== undefined) return `Shift ${correctionText(only)}`;
  return `${countOf(shifted, 'track')} shifted`;
}

export function trackDetail(tracks: readonly LoadedTrack[], zone: string): string {
  if (tracks.length === 0) return '';
  const start = Math.min(...tracks.map((track) => track.start));
  const end = Math.max(...tracks.map((track) => track.end));
  const metres = tracks.reduce((sum, track) => sum + track.metres, 0);
  return [when(start, end, zone), distanceText(metres), trackShiftText(tracks)]
    .filter(Boolean)
    .join(' · ');
}
