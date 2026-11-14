import { afterEach, describe, expect, it } from 'vitest';
import { preferences } from '#lib/stores/preferences.svelte.ts';
import { defaultKeysFor, hint, keysFor } from './labels';
import type { KeybindId } from './table';

describe('platform labels', () => {
  it.each<[KeybindId, string, string]>([
    ['undo', '⌘Z', 'Ctrl+Z'],
    ['redo', '⌘⇧Z', 'Ctrl+Shift+Z'],
    ['openMasks', '⇧M', 'Shift+M'],
    ['copyEdits', '⌘C', 'Ctrl+C'],
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

  describe('custom keys', () => {
    afterEach(() => preferences.reset());

    it('shows the custom keys and keeps the default for the note', () => {
      preferences.shortcuts = { autoAdjust: ['Mod+Shift+a'], help: ['F1'] };
      expect(keysFor('autoAdjust', true)).toBe('⌘⇧A');
      expect(defaultKeysFor('autoAdjust', true)).toBe('⌘U');
      expect(keysFor('help', false)).toBe('F1');
      expect(hint('Auto', 'autoAdjust', false)).toBe('Auto (Ctrl+Shift+A)');
    });

    it('drops the brackets for an unbound action', () => {
      preferences.shortcuts = { autoAdjust: [] };
      expect(hint('Auto', 'autoAdjust', false)).toBe('Auto');
    });
  });
});
