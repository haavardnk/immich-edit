import type { PlaceName } from '#lib/api/map.ts';

export interface LocationMove {
  id: string;
  lat: number;
  lon: number;
  place: PlaceName | null;
}

type Listener = (moves: readonly LocationMove[]) => void;

const listeners = new Set<Listener>();

export function locationsMoved(moves: readonly LocationMove[]): void {
  if (moves.length === 0) return;
  for (const listener of listeners) listener(moves);
}

export function onLocationsMoved(listener: Listener): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}
