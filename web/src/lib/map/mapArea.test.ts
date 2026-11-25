import type { MapMarker } from '#lib/api/map.ts';
import {
  sameView,
  boundsOf,
  formatView,
  inBox,
  markerQuery,
  markersIn,
  parseView,
  placeOfMarkers,
  readMapFilter,
  writeMapFilter
} from '#lib/map/mapArea.ts';
import { describe, expect, it } from 'vitest';

function marker(id: string, lat: number, lon: number, city: string | null = null): MapMarker {
  return { id, lat, lon, city, state: null, country: 'Norway' };
}

const BERGEN = marker('b', 60.39, 5.32, 'Bergen');
const VOSS = marker('v', 60.63, 6.42, 'Voss');
const FIJI = marker('f', -17.7, 178.1, 'Suva');

describe('map area', () => {
  it('keeps markers inside a box, across the antimeridian too', () => {
    const norway = { west: 5, south: 60, east: 7, north: 61 };
    expect(markersIn([BERGEN, VOSS, FIJI], norway).map((m) => m.id)).toEqual(['b', 'v']);
    const pacific = { west: 170, south: -30, east: -170, north: 0 };
    expect(inBox(FIJI, pacific)).toBe(true);
    expect(inBox(BERGEN, pacific)).toBe(false);
  });

  it('bounds every marker and names the commonest places', () => {
    expect(boundsOf([BERGEN, VOSS])).toEqual({
      west: 5.32,
      east: 6.42,
      south: 60.39,
      north: 60.63
    });
    expect(boundsOf([])).toBeNull();
    expect(placeOfMarkers([BERGEN, BERGEN, VOSS])).toBe('Bergen and Voss');
    expect(placeOfMarkers([marker('x', 1, 1)])).toBe('Norway');
  });

  it('round-trips a view and refuses a broken one', () => {
    const view = parseView(formatView({ lat: 60.39, lon: 5.32, zoom: 12 }));
    expect(view).toEqual({ lat: 60.39, lon: 5.32, zoom: 12 });
    for (const bad of [null, '60,5', '91,0,3', '0,0,30', 'a,b,c'])
      expect(parseView(bad)).toBeNull();
  });

  it('reads and writes the filters, ignoring malformed dates', () => {
    const params = new URLSearchParams('favorite=1&after=2026-10-01&before=tomorrow');
    const filter = readMapFilter(params);
    expect(filter).toEqual({ favorite: true, after: '2026-10-01', before: undefined });
    const written = new URLSearchParams('at=1,2,3');
    writeMapFilter(written, filter);
    expect(written.toString()).toBe('at=1%2C2%2C3&favorite=1&after=2026-10-01');
    expect(markerQuery(filter)).toEqual({
      favorite: true,
      taken: { from: '2026-10-01T00:00:00' }
    });
    expect(markerQuery({ before: '2026-10-31' })).toEqual({
      favorite: undefined,
      taken: { until: '2026-11-01T00:00:00' }
    });
    expect(markerQuery({}).taken).toBeUndefined();
  });

  it('treats views within URL rounding as the same', () => {
    const view = { lat: 60.39019, lon: 5.321, zoom: 16 };
    expect(sameView(view, { lat: 60.390194, lon: 5.320996, zoom: 16.004 })).toBe(true);
    expect(sameView(view, { ...view, zoom: 12 })).toBe(false);
    expect(sameView(view, { ...view, lat: 60.4 })).toBe(false);
  });
});
