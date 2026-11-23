import type { SavedPlace } from '#lib/api/places.ts';
import { PlacesStore } from '#lib/stores/places.svelte.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({
  markPlaceUsed: vi.fn(() => Promise.resolve()),
  rememberPlace: vi.fn((spot: { lat: number }) =>
    Promise.resolve([{ id: 'r1', used_at: 'now', ...spot }])
  )
}));

vi.mock('#lib/api/places.ts', () => ({
  markPlaceUsed: h.markPlaceUsed,
  rememberPlace: h.rememberPlace
}));

const HYTTA: SavedPlace = {
  id: 'hytta',
  name: 'Hytta',
  lat: 60.6289,
  lon: 6.4153,
  city: 'Voss',
  state: null,
  country: null,
  created_at: '2026-11-01T00:00:00Z',
  used_at: '2026-11-01T00:00:00Z'
};

beforeEach(() => {
  h.markPlaceUsed.mockClear();
  h.rememberPlace.mockClear();
});

describe('places store', () => {
  it('marks a saved place used when photos land on it', () => {
    const store = new PlacesStore();
    store.saved = [HYTTA];
    store.noteUse({ lat: 60.62891, lon: 6.41531, place: null });
    expect(h.markPlaceUsed).toHaveBeenCalledWith('hytta');
    expect(h.rememberPlace).not.toHaveBeenCalled();
    expect(store.saved[0]?.used_at).not.toBe(HYTTA.used_at);
  });

  it('remembers any other spot and hides recents that sit on a saved place', async () => {
    const store = new PlacesStore();
    store.saved = [HYTTA];
    store.noteUse({ lat: 59.9, lon: 10.7, place: { city: 'Oslo', state: null, country: null } });
    expect(h.rememberPlace).toHaveBeenCalledWith({
      lat: 59.9,
      lon: 10.7,
      city: 'Oslo',
      state: null,
      country: null
    });
    await vi.waitFor(() => expect(store.recent).toHaveLength(1));
    store.recent = [...store.recent, { ...HYTTA, id: 'r2', used_at: 'then' }];
    expect(store.freshRecent.map((spot) => spot.id)).toEqual(['r1']);
  });
});
