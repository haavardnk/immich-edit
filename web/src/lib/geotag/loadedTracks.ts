import { distanceText } from '#lib/geotag/spots.ts';
import type { TrackLine, TrackPoint } from '#lib/geotag/trackFile.ts';
import { TrackLog } from '#lib/geotag/trackLog.ts';
import { zoneName } from '#lib/metadata/captureLabels.ts';
import { correctionText } from '#lib/metadata/clockText.ts';
import { wallToInstant } from '#lib/metadata/photoTime.ts';
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
  utc: boolean;
  zone: string | null;
  offsetMs: number;
}

const HOUR = 3_600_000;

export function loadedTrack(fileName: string, line: TrackLine, id: number): LoadedTrack {
  const log = new TrackLog(line.segments);
  return {
    id: `track-${id}`,
    fileName,
    name: line.name,
    segments: line.segments,
    points: log.size,
    start: log.start,
    end: log.end,
    metres: log.lengthMetres(),
    utc: line.utc,
    zone: null,
    offsetMs: 0
  };
}

export function trackTime(track: LoadedTrack): (time: number) => number {
  const { zone, offsetMs } = track;
  if (!zone) return (time) => time + offsetMs;
  const deltas = new Map<number, number>();
  return (time) => {
    const hour = Math.floor(time / HOUR) * HOUR;
    let delta = deltas.get(hour);
    if (delta === undefined) {
      delta = wallToInstant(hour, zone) - hour;
      deltas.set(hour, delta);
    }
    return time + delta + offsetMs;
  };
}

function shiftedSegments(track: LoadedTrack): TrackPoint[][] {
  if (track.offsetMs === 0 && !track.zone) return track.segments;
  const timeOf = trackTime(track);
  return track.segments.map((points) =>
    points.map((point) => ({ ...point, time: timeOf(point.time) }))
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

export function trackChangeText(track: LoadedTrack): string {
  return [
    track.zone ? zoneName(track.zone) : '',
    track.offsetMs ? correctionText(track.offsetMs) : ''
  ]
    .filter(Boolean)
    .join(' · ');
}

export function trackShiftText(tracks: readonly LoadedTrack[]): string {
  const changes = new Set(tracks.map(trackChangeText));
  const changed = tracks.filter((track) => trackChangeText(track) !== '').length;
  if (changed === 0) return '';
  const [only] = changes;
  if (changes.size === 1 && only !== undefined) return only;
  return `${countOf(changed, 'track')} changed`;
}

export function trackDetail(tracks: readonly LoadedTrack[], zone: string): string {
  if (tracks.length === 0) return '';
  const start = Math.min(...tracks.map((track) => trackTime(track)(track.start)));
  const end = Math.max(...tracks.map((track) => trackTime(track)(track.end)));
  const metres = tracks.reduce((sum, track) => sum + track.metres, 0);
  return [when(start, end, zone), distanceText(metres), trackShiftText(tracks)]
    .filter(Boolean)
    .join(' · ');
}
