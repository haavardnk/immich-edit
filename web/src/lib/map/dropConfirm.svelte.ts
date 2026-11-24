import { reverseGeocode, type PlaceName } from '#lib/api/map.ts';
import type { SavedPlace } from '#lib/api/places.ts';
import { locationOfSpot } from '#lib/geotag/spots.ts';

export type DropVerb = 'move' | 'place';

export interface PendingDrop {
  ids: readonly string[];
  lat: number;
  lon: number;
  verb: DropVerb;
  pin: boolean;
  saved: SavedPlace | null;
  place: PlaceName | null;
  named: boolean;
}

export function dropQuestion(drop: Pick<PendingDrop, 'ids' | 'verb' | 'saved'>): string {
  const count = drop.ids.length === 1 ? 'this photo' : `${drop.ids.length} photos`;
  const where = drop.saved ? `at ${drop.saved.name}` : 'here';
  return `${drop.verb === 'move' ? 'Move' : 'Place'} ${count} ${where}?`;
}

export function placeLine(place: PlaceName | null): string | null {
  const parts = [place?.city, place?.state, place?.country].filter(
    (part): part is string => typeof part === 'string' && part !== ''
  );
  return parts.length > 0 ? parts.join(', ') : null;
}

export class DropConfirm {
  pending = $state.raw<PendingDrop | null>(null);
  #settle: ((place: PlaceName | null | false) => void) | null = null;

  ask(
    drop: Omit<PendingDrop, 'place' | 'named' | 'saved'> & { saved?: SavedPlace | null }
  ): Promise<PlaceName | null | false> {
    this.#settle?.(false);
    const saved = drop.saved ?? null;
    const known = saved ? locationOfSpot(saved).place : null;
    const pending: PendingDrop = { ...drop, saved, place: known, named: saved !== null };
    this.pending = pending;
    const settled = new Promise<PlaceName | null | false>((resolve) => {
      this.#settle = resolve;
    });
    if (saved) return settled;
    reverseGeocode(drop.lat, drop.lon)
      .then((place) => {
        if (this.pending === pending) this.pending = { ...pending, place, named: true };
      })
      .catch(() => {
        if (this.pending === pending) this.pending = { ...pending, named: true };
      });
    return settled;
  }

  confirm = (): void => this.#finish(this.pending?.place ?? null);

  cancel = (): void => this.#finish(false);

  #finish(result: PlaceName | null | false): void {
    const settle = this.#settle;
    this.#settle = null;
    this.pending = null;
    settle?.(result);
  }
}
