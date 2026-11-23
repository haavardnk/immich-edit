import type { RecentPlace, SavedPlace } from '#lib/api/places.ts';
import { parseCoordinates } from '#lib/geotag/coordinates.ts';
import {
  distanceLabel,
  matchesPlace,
  placeSections,
  spotLine,
  type SuggestionInput
} from '#lib/geotag/placeSuggestions.ts';
import { describe, expect, it } from 'vitest';

function saved(id: string, name: string, lat: number, lon: number, city: string): SavedPlace {
  return {
    id,
    name,
    lat,
    lon,
    city,
    state: 'Vestland',
    country: 'Norway',
    created_at: '2026-11-01T00:00:00Z',
    used_at: '2026-11-01T00:00:00Z'
  };
}

const HOME = saved('home', 'Home', 60.387, 5.329, 'Bergen');
const HYTTA = saved('hytta', 'Hytta', 60.6289, 6.4153, 'Voss');
const FLAM: RecentPlace = {
  id: 'flam',
  lat: 60.8628,
  lon: 7.1135,
  city: 'Flåm',
  state: 'Vestland',
  country: 'Norway',
  used_at: '2026-11-20T00:00:00Z'
};

function sections(input: Partial<SuggestionInput>) {
  const built = placeSections({
    query: '',
    direct: null,
    saved: [HOME, HYTTA],
    recent: [FLAM],
    found: [],
    ...input
  });
  return Object.fromEntries(
    built.map((section) => [section.id, section.items.map((item) => item.place)])
  );
}

describe('place suggestions', () => {
  it('lists saved then recent places before anything is typed', () => {
    const idle = sections({});
    expect(idle.saved?.map((place) => place.name)).toEqual(['Home', 'Hytta']);
    expect(idle.saved?.[0]?.detail).toBe('Bergen, Vestland, Norway');
    expect(idle.recent).toEqual([
      { name: 'Flåm', detail: 'Vestland, Norway', lat: 60.8628, lon: 7.1135 }
    ]);
    expect(idle.go).toEqual([]);
  });

  it('matches saved places by name or place and drops recents while typing', () => {
    const typed = sections({
      query: 'VOS',
      found: [{ name: 'Voss', detail: 'x', lat: 1, lon: 2 }]
    });
    expect(typed.saved?.map((place) => place.name)).toEqual(['Hytta']);
    expect(typed.recent).toEqual([]);
    expect(typed.found?.map((place) => place.name)).toEqual(['Voss']);
    expect(matchesPlace(HOME, 'berg')).toBe(true);
    expect(matchesPlace(HOME, 'oslo')).toBe(false);
  });

  it('puts typed coordinates first with the saved places nearby', () => {
    const near = sections({
      query: '60.3913, 5.3221',
      direct: parseCoordinates('60.3913, 5.3221')
    });
    expect(near.go?.[0]).toMatchObject({ name: '60.3913° N, 5.3221° E', detail: 'Coordinates' });
    expect(near.saved?.map((place) => place.detail)).toEqual([
      'Bergen, Vestland, Norway · 610 m away'
    ]);
  });

  it('formats distances and place lines', () => {
    expect(distanceLabel(4)).toBe('10 m away');
    expect(distanceLabel(1240)).toBe('1.2 km away');
    expect(distanceLabel(48_700)).toBe('49 km away');
    expect(spotLine({ city: 'Voss', state: null, country: 'Norway' }, 'Voss')).toBe('Norway');
  });
});
