import { chordMatches } from './chord';
import { effectiveChords } from './effective';
import { KEYBINDS, type KeybindContext, type KeybindId } from './table';

export function isKeybind(e: KeyboardEvent, id: KeybindId): boolean {
  return effectiveChords(id).some((chord) => chordMatches(e, chord));
}

export function matchKeybinds(e: KeyboardEvent, contexts: readonly KeybindContext[]): KeybindId[] {
  return KEYBINDS.filter(
    (bind) => bind.contexts.some((c) => contexts.includes(c)) && isKeybind(e, bind.id)
  ).map((bind) => bind.id);
}
