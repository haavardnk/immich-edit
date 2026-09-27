import { chordMatches, parseChord, type Chord } from './chord';
import { KEYBINDS, type KeybindContext, type KeybindId } from './table';

const CHORDS = new Map<string, Chord[]>(KEYBINDS.map((b) => [b.id, b.keys.map(parseChord)]));

export function isKeybind(e: KeyboardEvent, id: KeybindId): boolean {
  return (CHORDS.get(id) ?? []).some((chord) => chordMatches(e, chord));
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
