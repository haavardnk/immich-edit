import type { GeoPoint } from '#lib/browse/info/photoInfo.ts';
import type { PhotoMatch, PhotoOutcome } from '#lib/geotag/match.ts';
import { distanceMetres, SAME_SPOT_METRES } from '#lib/geotag/spots.ts';

export interface Reference {
  at: GeoPoint;
  on: GeoPoint | null;
  metres: number | null;
  moved: boolean;
}

export interface MarkPin extends GeoPoint {
  id: string;
}

export interface ReferencePin extends MarkPin {
  moved: boolean;
}

export interface Connector {
  id: string;
  from: GeoPoint;
  to: GeoPoint;
}

export interface TrackMarks {
  pins: MarkPin[];
  references: ReferencePin[];
  connectors: Connector[];
}

function pointOf(spot: GeoPoint): GeoPoint {
  return { lat: spot.lat, lon: spot.lon };
}

export function referenceOf(outcome: PhotoOutcome): Reference | null {
  if (outcome.kind === 'kept') {
    const on = outcome.track.kind === 'placed' ? pointOf(outcome.track) : null;
    return { at: outcome.at, on, metres: on ? distanceMetres(outcome.at, on) : null, moved: false };
  }
  if (outcome.kind === 'placed' && outcome.from) {
    const on = pointOf(outcome);
    return { at: outcome.from, on, metres: distanceMetres(outcome.from, on), moved: true };
  }
  return null;
}

export function pinSpot(outcome: PhotoOutcome | undefined): GeoPoint | null {
  if (outcome?.kind === 'placed') return pointOf(outcome);
  if (outcome?.kind === 'kept') return outcome.at;
  return null;
}

export function trackMarks(matches: readonly PhotoMatch[]): TrackMarks {
  const marks: TrackMarks = { pins: [], references: [], connectors: [] };
  for (const { id, outcome } of matches) {
    if (outcome.kind === 'placed') marks.pins.push({ id, lat: outcome.lat, lon: outcome.lon });
    const reference = referenceOf(outcome);
    if (!reference) continue;
    marks.references.push({ id, ...reference.at, moved: reference.moved });
    const far = reference.metres !== null && reference.metres > SAME_SPOT_METRES;
    if (reference.on && (reference.moved || far)) {
      marks.connectors.push({ id, from: reference.at, to: reference.on });
    }
  }
  return marks;
}

export function focusLink(outcome: PhotoOutcome | undefined): Connector | null {
  const reference = outcome ? referenceOf(outcome) : null;
  return reference?.on ? { id: 'focus', from: reference.at, to: reference.on } : null;
}
