import { getContext, setContext } from 'svelte';

export type TileOpen = 'editor' | 'loupe';

const KEY = Symbol('tileOpen');

export function provideTileOpen(open: TileOpen): void {
  setContext(KEY, open);
}

export function tileOpen(): TileOpen {
  return getContext<TileOpen | undefined>(KEY) ?? 'editor';
}
