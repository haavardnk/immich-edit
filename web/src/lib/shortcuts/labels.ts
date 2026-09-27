import { isMac } from '$lib/utils/platform';
import { formatChord } from './chord';
import { KEYBINDS, type Keybind, type KeybindId } from './table';

export function keysFor(id: KeybindId, mac: boolean = isMac): string {
  const bind = (KEYBINDS as readonly Keybind[]).find((b) => b.id === id);
  if (!bind) return '';
  if (bind.display) return typeof bind.display === 'string' ? bind.display : bind.display(mac);
  return bind.keys.map((spec) => formatChord(spec, mac)).join(' / ');
}

export function hint(label: string, id: KeybindId, mac: boolean = isMac): string {
  return `${label} (${keysFor(id, mac)})`;
}
