import { reverseGeocode, type PlaceName } from '#lib/api/map.ts';
import type { SavedPlace } from '#lib/api/places.ts';
import type { GeoPoint } from '#lib/browse/info/photoInfo.ts';
import { places } from '#lib/stores/places.svelte.ts';

export type PlaceEdit =
  { kind: 'move'; place: SavedPlace; at: GeoPoint } | { kind: 'create'; at: GeoPoint };

export const PLACE_ZOOM = 14;

export interface PlaceMenuAt {
  x: number;
  y: number;
  at: GeoPoint;
}

export interface MapPlacesHost {
  hover: (place: SavedPlace | null) => void;
  show: (place: SavedPlace) => void;
  goTo: (at: GeoPoint) => void;
  create: (at?: GeoPoint, name?: string) => void;
  move: (place: SavedPlace) => void;
  photos: (place: SavedPlace) => void;
}

export class PlaceEditor {
  #cardId = $state<string | null>(null);
  readonly card = $derived(
    this.#cardId ? (places.saved.find((place) => place.id === this.#cardId) ?? null) : null
  );
  edit = $state.raw<PlaceEdit | null>(null);
  menu = $state.raw<PlaceMenuAt | null>(null);
  renaming = $state.raw<SavedPlace | null>(null);
  name = $state('');
  where = $state.raw<PlaceName | null>(null);
  busy = $state(false);
  #lookup = 0;

  open = (place: SavedPlace): void => {
    if (this.edit) return;
    this.menu = null;
    this.#cardId = place.id;
  };

  close = (): void => {
    this.#cardId = null;
  };

  startMove = (place: SavedPlace): void => {
    this.#cardId = null;
    this.menu = null;
    this.where = { city: place.city, state: place.state, country: place.country };
    this.edit = { kind: 'move', place, at: { lat: place.lat, lon: place.lon } };
  };

  startCreate = (at: GeoPoint, name = ''): void => {
    this.#cardId = null;
    this.menu = null;
    this.name = name;
    this.where = null;
    this.edit = { kind: 'create', at };
    void this.#look(at, name === '');
  };

  place = (at: GeoPoint): void => {
    const edit = this.edit;
    if (!edit) return;
    this.edit = { ...edit, at };
    void this.#look(at, false);
  };

  cancel = (): void => {
    this.edit = null;
    this.busy = false;
  };

  done = async (): Promise<void> => {
    const edit = this.edit;
    if (!edit || this.busy) return;
    const name = this.name.trim();
    if (edit.kind === 'create' && !name) return;
    this.busy = true;
    try {
      const named = await reverseGeocode(edit.at.lat, edit.at.lon).catch(() => null);
      const location = { ...edit.at, place: named };
      const saved =
        edit.kind === 'move'
          ? await places.move(edit.place, location)
          : (await places.create(name, location)) !== null;
      if (saved && this.edit === edit) this.edit = null;
    } finally {
      this.busy = false;
    }
  };

  #look = async (at: GeoPoint, fillName: boolean): Promise<void> => {
    const ticket = ++this.#lookup;
    const named = await reverseGeocode(at.lat, at.lon).catch(() => null);
    if (ticket !== this.#lookup) return;
    this.where = named;
    if (fillName && !this.name.trim()) this.name = named?.city ?? '';
  };
}
