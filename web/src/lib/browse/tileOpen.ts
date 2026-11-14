import type { ClickTarget } from '#lib/api/preferences.ts';
import { preferences } from '#lib/stores/preferences.svelte.ts';
import { getContext, setContext } from 'svelte';

const KEY = Symbol('tileOpen');

export function provideTileOpen(open: ClickTarget): void {
  setContext(KEY, open);
}

export function tileOpen(): () => ClickTarget {
  const fixed = getContext<ClickTarget | undefined>(KEY);
  return () => fixed ?? preferences.clickTarget;
}
