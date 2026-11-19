import { preferences } from '#lib/stores/preferences.svelte.ts';
import { isMac } from '#lib/utils/platform.ts';
import { formatChord } from './chord';
import { bindOf, isCustomised } from './custom';
import { effectiveKeys } from './effective';
import type { KeybindId } from './table';

function chordList(keys: readonly string[], mac: boolean): string {
  return keys.map((spec) => formatChord(spec, mac)).join(' / ');
}

export function defaultKeysFor(id: string, mac: boolean = isMac): string {
  const bind = bindOf(id);
  if (!bind) return '';
  if (bind.display) return typeof bind.display === 'string' ? bind.display : bind.display(mac);
  return chordList(bind.keys, mac);
}

export function keysFor(id: KeybindId, mac: boolean = isMac): string {
  if (!isCustomised(id, preferences.shortcuts)) return defaultKeysFor(id, mac);
  return chordList(effectiveKeys(id), mac);
}

export function hint(label: string, id: KeybindId, mac: boolean = isMac): string {
  const keys = keysFor(id, mac);
  return keys ? `${label} (${keys})` : label;
}
