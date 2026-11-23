import {
  clearRecentPlaces,
  createPlace,
  deletePlace,
  listPlaces,
  markPlaceUsed,
  rememberPlace,
  updatePlace,
  type RecentPlace,
  type SavedPlace,
  type SavedPlaceInput
} from '#lib/api/places.ts';
import type { GeoLocation } from '#lib/browse/locationOps.ts';
import { sameSpot, spotOf } from '#lib/geotag/spots.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

function byUse(a: SavedPlace, b: SavedPlace): number {
  return (
    b.used_at.localeCompare(a.used_at) ||
    a.name.localeCompare(b.name, undefined, { sensitivity: 'base' })
  );
}

export function placeInput(place: SavedPlace, name: string = place.name): SavedPlaceInput {
  return {
    name,
    lat: place.lat,
    lon: place.lon,
    city: place.city,
    state: place.state,
    country: place.country
  };
}

export class PlacesStore {
  saved = $state.raw<SavedPlace[]>([]);
  recent = $state.raw<RecentPlace[]>([]);
  loaded = $state(false);
  #loading: Promise<void> | null = null;

  readonly freshRecent = $derived(
    this.recent.filter((spot) => !this.saved.some((place) => sameSpot(place, spot)))
  );

  ensure = (): Promise<void> => {
    this.#loading ??= this.#load();
    return this.#loading;
  };

  #load = async (): Promise<void> => {
    try {
      const places = await listPlaces();
      this.saved = [...places.saved].sort(byUse);
      this.recent = places.recent;
      this.loaded = true;
    } catch (e) {
      this.#loading = null;
      toasts.fail('Could not load saved places', e);
    }
  };

  #put(place: SavedPlace): void {
    this.saved = [...this.saved.filter((p) => p.id !== place.id), place].sort(byUse);
  }

  savedAt = (location: Pick<GeoLocation, 'lat' | 'lon'>): SavedPlace | null =>
    this.saved.find((place) => sameSpot(place, location)) ?? null;

  create = async (name: string, location: GeoLocation): Promise<SavedPlace | null> => {
    await this.ensure();
    try {
      const place = await createPlace({ name, ...spotOf(location) });
      this.#put(place);
      toasts.push('success', `Saved ${place.name}`);
      return place;
    } catch (e) {
      toasts.fail('Could not save the place', e);
      return null;
    }
  };

  update = async (place: SavedPlace, input: SavedPlaceInput): Promise<boolean> => {
    await this.ensure();
    try {
      this.#put(await updatePlace(place.id, input));
      return true;
    } catch (e) {
      toasts.fail('Could not update the place', e);
      return false;
    }
  };

  rename = (place: SavedPlace, name: string): Promise<boolean> =>
    this.update(place, placeInput(place, name));

  remove = async (place: SavedPlace): Promise<void> => {
    await this.ensure();
    this.saved = this.saved.filter((p) => p.id !== place.id);
    try {
      await deletePlace(place.id);
      toasts.push('success', `Removed ${place.name}`, {
        action: { label: 'Undo', run: () => void this.#restore(place) }
      });
    } catch (e) {
      this.#put(place);
      toasts.fail('Could not remove the place', e);
    }
  };

  #restore = async (place: SavedPlace): Promise<void> => {
    try {
      this.#put(await createPlace(placeInput(place)));
    } catch (e) {
      toasts.fail('Could not restore the place', e);
    }
  };

  noteUse = (location: GeoLocation): void => {
    const place = this.savedAt(location);
    if (place) {
      this.#put({ ...place, used_at: new Date().toISOString() });
      markPlaceUsed(place.id).catch(() => undefined);
      return;
    }
    rememberPlace(spotOf(location))
      .then((recent) => {
        this.recent = recent;
      })
      .catch(() => undefined);
  };

  clearRecent = async (): Promise<void> => {
    const before = this.recent;
    this.recent = [];
    try {
      await clearRecentPlaces();
    } catch (e) {
      this.recent = before;
      toasts.fail('Could not clear recent places', e);
    }
  };
}

export const places = new PlacesStore();
