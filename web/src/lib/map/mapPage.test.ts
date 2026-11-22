import { MapPageState } from '#lib/map/mapPage.svelte.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({ getMapMarkers: vi.fn() }));

vi.mock('#lib/api/map.ts', () => ({ getMapMarkers: h.getMapMarkers }));
vi.mock('#lib/map/mapFeed.svelte.ts', () => ({ MapFeed: class {} }));
vi.mock('#lib/map/unplacedFeed.svelte.ts', () => ({
  UnplacedFeed: class {
    remove = vi.fn();
  }
}));

const OSLO = { id: 'a', lat: 59.91, lon: 10.75, city: 'Oslo', state: null, country: 'Norway' };
const BERGEN = { city: 'Bergen', state: 'Vestland', country: 'Norway' };

beforeEach(() => {
  h.getMapMarkers.mockReset();
  h.getMapMarkers.mockResolvedValue([OSLO]);
});

describe('map page state', () => {
  it('moves a known marker in place without reloading', async () => {
    const state = new MapPageState({});
    await state.load();
    state.move([{ id: 'a', lat: 60.39, lon: 5.32, place: BERGEN }]);
    expect(state.markers).toEqual([{ id: 'a', lat: 60.39, lon: 5.32, ...BERGEN }]);
    expect(h.getMapMarkers).toHaveBeenCalledOnce();
  });

  it('reloads when a photo without a marker gets a location', async () => {
    const state = new MapPageState({});
    await state.load();
    state.move([{ id: 'new', lat: 1, lon: 2, place: null }]);
    expect(h.getMapMarkers).toHaveBeenCalledTimes(2);
  });
});
