import { preferences } from '#lib/stores/preferences.svelte.ts';
import { parseChord, type Chord } from './chord';
import { keysOf, type ShortcutMap } from './custom';
import { KEYBINDS } from './table';

let built: { custom: ShortcutMap; chords: ReadonlyMap<string, readonly Chord[]> } | null = null;

export function effectiveKeys(id: string): readonly string[] {
  return keysOf(id, preferences.shortcuts);
}

export function effectiveChords(id: string): readonly Chord[] {
  const custom = preferences.shortcuts;
  if (built?.custom !== custom) {
    const chords = new Map(
      KEYBINDS.map((bind) => [bind.id, keysOf(bind.id, custom).map(parseChord)])
    );
    built = { custom, chords };
  }
  return built.chords.get(id) ?? [];
}
