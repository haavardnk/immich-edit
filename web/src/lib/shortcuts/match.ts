import { chordMatches } from './chord';
import { effectiveChords } from './effective';
import { KEYBINDS, type KeybindContext, type KeybindId } from './table';

export function isKeybind(e: KeyboardEvent, id: KeybindId): boolean {
  return effectiveChords(id).some((chord) => chordMatches(e, chord));
}

export function matchKeybind(
  e: KeyboardEvent,
  contexts: readonly KeybindContext[]
): KeybindId | null {
  for (const bind of KEYBINDS) {
    if (!bind.contexts.some((c) => contexts.includes(c))) continue;
    if (isKeybind(e, bind.id)) return bind.id;
  }
  return null;
}
