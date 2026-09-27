import { describe, expect, it } from 'vitest';
import { matchKeybind } from './match';
import { KEYBINDS, type Keybind, type KeybindContext } from './table';

const CONTEXTS: KeybindContext[] = [
  'global',
  'grid',
  'loupe',
  'compare',
  'survey',
  'editor',
  'geometry',
  'masks',
  'retouch'
];

describe('registry integrity', () => {
  const all = KEYBINDS as readonly Keybind[];

  it.each(CONTEXTS)('has no colliding chords within %s', (context) => {
    const seen = new Map<string, string>();
    for (const bind of all) {
      if (!bind.contexts.includes(context)) continue;
      for (const spec of bind.keys) {
        const prev = seen.get(spec);
        expect(prev, `${spec}: ${prev} vs ${bind.id}`).toBeUndefined();
        seen.set(spec, bind.id);
      }
    }
  });

  it('has unique ids', () => {
    const ids = all.map((b) => b.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it('gives every keyless bind a display and no match', () => {
    const shift = {
      key: 'Shift',
      code: '',
      metaKey: false,
      ctrlKey: false,
      shiftKey: false,
      altKey: false
    } as KeyboardEvent;
    const keyless = all.filter((bind) => bind.keys.length === 0);
    expect(keyless.length).toBeGreaterThan(0);
    for (const bind of keyless) {
      expect(bind.display, bind.id).toBeTruthy();
      expect(matchKeybind(shift, bind.contexts)).not.toBe(bind.id);
    }
  });
});
