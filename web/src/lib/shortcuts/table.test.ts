import { describe, expect, it, vi } from 'vitest';
import { CONTEXT_STACKS } from './context';
import { matchKeybind } from './match';
import { KEYBINDS, type Keybind } from './table';

vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/') } }));

const SHARED_BY_DESIGN = /^(Escape|(Shift\+)?Arrow\w+)$/;

describe('registry integrity', () => {
  const all = KEYBINDS as readonly Keybind[];

  it.each(Object.entries(CONTEXT_STACKS))('has no colliding chords in the %s stack', (_, stack) => {
    const seen = new Map<string, string>();
    for (const bind of all) {
      if (!bind.contexts.some((c) => (stack as readonly string[]).includes(c))) continue;
      for (const spec of bind.keys.filter((k) => !SHARED_BY_DESIGN.test(k))) {
        const chord = spec.toLowerCase();
        const prev = seen.get(chord);
        expect(prev, `${spec}: ${prev} vs ${bind.id}`).toBeUndefined();
        seen.set(chord, bind.id);
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
      altKey: false,
      getModifierState: () => false
    } as unknown as KeyboardEvent;
    const keyless = all.filter((bind) => bind.keys.length === 0);
    expect(keyless.length).toBeGreaterThan(0);
    for (const bind of keyless) {
      expect(bind.display, bind.id).toBeTruthy();
      expect(matchKeybind(shift, bind.contexts)).not.toBe(bind.id);
    }
  });
});
