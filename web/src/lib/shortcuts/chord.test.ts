import { describe, expect, it } from 'vitest';
import { formatChord, keyLabel } from './chord';

describe('chord labels', () => {
  it('formats bare chords', () => {
    expect(formatChord('Mod+Shift+e', true)).toBe('⌘⇧E');
    expect(formatChord('Mod+Shift+e', false)).toBe('Ctrl+Shift+E');
  });

  it('labels a code chord by its digit', () => {
    expect(formatChord('Shift+Digit3', false)).toBe('Shift+3');
  });

  it.each<[string, string, string]>([
    ['Alt', '⌥', 'Alt'],
    ['Shift', '⇧', 'Shift'],
    ['Enter', 'Return', 'Enter'],
    ['Escape', 'Esc', 'Esc']
  ])('keyLabel(%s) renders per platform', (key, mac, pc) => {
    expect(keyLabel(key, true)).toBe(mac);
    expect(keyLabel(key, false)).toBe(pc);
  });
});
