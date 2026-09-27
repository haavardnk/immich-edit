import { describe, expect, it } from 'vitest';
import { keysFor } from './labels';
import type { KeybindId } from './table';

describe('platform labels', () => {
  it.each<[KeybindId, string, string]>([
    ['undo', '⌘Z', 'Ctrl+Z'],
    ['redo', '⌘⇧Z', 'Ctrl+Shift+Z'],
    ['fullscreen', '⇧F', 'Shift+F'],
    ['editorEscape', 'Esc', 'Esc'],
    ['maskDelete', '⌫ / ⌦', 'Backspace / Del'],
    ['loupeNav', '← / →', '← / →'],
    ['togglePanels', 'Tab', 'Tab']
  ])('%s renders per platform', (id, mac, pc) => {
    expect(keysFor(id, true)).toBe(mac);
    expect(keysFor(id, false)).toBe(pc);
  });

  it('honors display overrides', () => {
    expect(keysFor('rate', true)).toBe(keysFor('rate', false));
  });
});
