import { gpx, kml, tcx } from '@tmcw/togeojson';
import type { Feature, Geometry, Position } from 'geojson';

export interface TrackPoint {
  lat: number;
  lon: number;
  time: number;
}

export interface TrackLine {
  name: string | null;
  segments: TrackPoint[][];
  utc: boolean;
}

export interface ParsedTrack {
  lines: TrackLine[];
  untimed: number;
}

export type TrackFormat = 'gpx' | 'kml' | 'tcx';

export const TRACK_EXTENSIONS = ['.gpx', '.kml', '.tcx'] as const;

const READERS = { gpx, kml, tcx } as const;

export function trackFormat(fileName: string): TrackFormat | null {
  const match = /\.(gpx|kml|tcx)$/i.exec(fileName.trim());
  return match?.[1] ? (match[1].toLowerCase() as TrackFormat) : null;
}

function timesOf(feature: Feature<Geometry | null>): unknown {
  const props = feature.properties as { coordinateProperties?: { times?: unknown } } | null;
  return props?.coordinateProperties?.times;
}

function segmentsOf(feature: Feature<Geometry | null>): [Position[], unknown[]][] {
  const geometry = feature.geometry;
  const times = timesOf(feature);
  if (geometry?.type === 'LineString') {
    return [[geometry.coordinates, Array.isArray(times) ? times : []]];
  }
  if (geometry?.type === 'MultiLineString') {
    return geometry.coordinates.map((line, index) => {
      const own = Array.isArray(times) ? times[index] : undefined;
      return [line, Array.isArray(own) ? own : []];
    });
  }
  return [];
}

function parseTime(raw: unknown): number {
  if (typeof raw !== 'string' && typeof raw !== 'number') return Number.NaN;
  if (typeof raw === 'number') return raw;
  const iso = /[zZ]|[+-]\d\d:?\d\d$/.test(raw) ? raw : `${raw}Z`;
  return Date.parse(iso);
}

function utcTime(raw: unknown): boolean {
  return typeof raw !== 'string' || !/[+-]\d\d:?\d\d$/.test(raw) || /[+-]00:?00$/.test(raw);
}

function isRoute(feature: Feature<Geometry | null>): boolean {
  return (feature.properties as { _gpxType?: string } | null)?._gpxType === 'rte';
}

export function readTrack(document: Document, format: TrackFormat): ParsedTrack {
  let untimed = 0;
  const lines: TrackLine[] = [];
  for (const feature of READERS[format](document).features) {
    if (isRoute(feature)) continue;
    let utc = true;
    const segments = segmentsOf(feature)
      .map(([coordinates, times]) => {
        const points: TrackPoint[] = [];
        coordinates.forEach(([lon, lat], index) => {
          const time = parseTime(times[index]);
          if (lat === undefined || lon === undefined) return;
          if (!Number.isFinite(time)) {
            untimed += 1;
            return;
          }
          if (!utcTime(times[index])) utc = false;
          points.push({ lat, lon, time });
        });
        return points.sort((a, b) => a.time - b.time);
      })
      .filter((points) => points.length > 0);
    if (segments.length === 0) continue;
    const name = (feature.properties as { name?: unknown } | null)?.name;
    lines.push({
      name: typeof name === 'string' && name.trim() ? name.trim() : null,
      segments,
      utc
    });
  }
  return { lines, untimed };
}

export function parseTrackFile(source: string, format: TrackFormat): ParsedTrack {
  const document = new DOMParser().parseFromString(source, 'application/xml');
  if (document.getElementsByTagName('parsererror').length > 0) return { lines: [], untimed: 0 };
  return readTrack(document, format);
}
