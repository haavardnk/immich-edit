import type { Place } from '#lib/api/map.ts';
import type { RecentPlace, SavedPlace, Spot } from '#lib/api/places.ts';
import { coordinateLabel } from '#lib/browse/info/photoInfo.ts';
import type { ParsedCoordinates } from '#lib/geotag/coordinates.ts';
import { distanceMetres } from '#lib/geotag/spots.ts';
import { mdiCrosshairsGps, mdiHistory, mdiLinkVariant, mdiStar } from '@mdi/js';

export type SectionId = 'go' | 'saved' | 'recent' | 'found';

export interface PlaceSuggestion {
  key: string;
  icon: string | null;
  saved: boolean;
  trail: string | null;
  place: Place;
}

export interface PlaceSection {
  id: SectionId;
  title: string;
  heading: string | null;
  items: PlaceSuggestion[];
}

export interface SuggestionInput {
  query: string;
  direct: ParsedCoordinates | null;
  saved: readonly SavedPlace[];
  recent: readonly RecentPlace[];
  found: readonly Place[];
}

const IDLE_SAVED = 8;
const TYPED_SAVED = 5;
const IDLE_RECENT = 5;
const NEARBY_METRES = 2000;

export function matchesPlace(place: SavedPlace, query: string): boolean {
  const needle = query.trim().toLocaleLowerCase();
  if (needle === '') return true;
  return [place.name, place.city, place.state, place.country].some((part) =>
    part?.toLocaleLowerCase().includes(needle)
  );
}

export function spotLine(spot: Pick<Spot, 'city' | 'state' | 'country'>, skip?: string): string {
  return [spot.city, spot.state, spot.country]
    .filter((part): part is string => !!part && part !== skip)
    .join(', ');
}

export function distanceLabel(metres: number): string {
  if (metres < 1000) return `${Math.max(10, Math.round(metres / 10) * 10)} m away`;
  return `${(metres / 1000).toFixed(metres < 10_000 ? 1 : 0)} km away`;
}

function savedSuggestion(place: SavedPlace, near: ParsedCoordinates | null): PlaceSuggestion {
  const line = spotLine(place);
  const away = near ? distanceLabel(distanceMetres(near, place)) : null;
  return {
    key: `saved:${place.id}`,
    icon: mdiStar,
    saved: true,
    trail: null,
    place: {
      name: place.name,
      detail: [line, away].filter(Boolean).join(' · '),
      lat: place.lat,
      lon: place.lon
    }
  };
}

function recentSuggestion(spot: RecentPlace): PlaceSuggestion {
  const name = spot.city ?? spot.state ?? spot.country ?? coordinateLabel(spot.lat, spot.lon);
  return {
    key: `recent:${spot.id}`,
    icon: mdiHistory,
    saved: false,
    trail: null,
    place: { name, detail: spotLine(spot, name), lat: spot.lat, lon: spot.lon }
  };
}

function directSuggestion(point: ParsedCoordinates): PlaceSuggestion {
  return {
    key: `go:${point.lat},${point.lon}`,
    icon: point.source === 'link' ? mdiLinkVariant : mdiCrosshairsGps,
    saved: false,
    trail: 'Go to',
    place: {
      name: coordinateLabel(point.lat, point.lon),
      detail: point.source === 'link' ? 'From the pasted map link' : 'Coordinates',
      lat: point.lat,
      lon: point.lon
    }
  };
}

function savedMatches(input: SuggestionInput): SavedPlace[] {
  const { direct, saved } = input;
  const typed = input.query.trim();
  if (direct) {
    return saved
      .map((place) => ({ place, metres: distanceMetres(direct, place) }))
      .filter(({ metres }) => metres <= NEARBY_METRES)
      .sort((a, b) => a.metres - b.metres)
      .slice(0, TYPED_SAVED)
      .map(({ place }) => place);
  }
  if (typed === '') return saved.slice(0, IDLE_SAVED);
  return saved.filter((place) => matchesPlace(place, typed)).slice(0, TYPED_SAVED);
}

export function placeSections(input: SuggestionInput): PlaceSection[] {
  const idle = input.query.trim() === '';
  return [
    {
      id: 'go',
      title: 'Go to',
      heading: null,
      items: input.direct ? [directSuggestion(input.direct)] : []
    },
    {
      id: 'saved',
      title: 'Saved places',
      heading: 'Saved',
      items: savedMatches(input).map((place) => savedSuggestion(place, input.direct))
    },
    {
      id: 'recent',
      title: 'Recent places',
      heading: 'Recent',
      items: idle ? input.recent.slice(0, IDLE_RECENT).map(recentSuggestion) : []
    },
    {
      id: 'found',
      title: 'Places',
      heading: null,
      items: input.found.map((place) => ({
        key: `found:${place.lat},${place.lon},${place.name}`,
        icon: null,
        saved: false,
        trail: null,
        place
      }))
    }
  ];
}
