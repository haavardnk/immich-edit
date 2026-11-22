import { reverseGeocode, type PlaceName } from '#lib/api/map.ts';

export type DropVerb = 'move' | 'place';

export interface PendingDrop {
  ids: readonly string[];
  lat: number;
  lon: number;
  verb: DropVerb;
  pin: boolean;
  place: PlaceName | null;
  named: boolean;
}

export function dropQuestion(drop: Pick<PendingDrop, 'ids' | 'verb'>): string {
  const count = drop.ids.length === 1 ? 'this photo' : `${drop.ids.length} photos`;
  return `${drop.verb === 'move' ? 'Move' : 'Place'} ${count} here?`;
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

  ask(drop: Omit<PendingDrop, 'place' | 'named'>): Promise<PlaceName | null | false> {
    this.#settle?.(false);
    const pending: PendingDrop = { ...drop, place: null, named: false };
    this.pending = pending;
    reverseGeocode(drop.lat, drop.lon)
      .then((place) => {
        if (this.pending === pending) this.pending = { ...pending, place, named: true };
      })
      .catch(() => {
        if (this.pending === pending) this.pending = { ...pending, named: true };
      });
    return new Promise((resolve) => {
      this.#settle = resolve;
    });
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
