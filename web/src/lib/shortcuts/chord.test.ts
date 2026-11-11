import { describe, expect, it } from 'vitest';
import { chordMatches, chordSpec, formatChord, keyLabel, parseChord } from './chord';

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

function press(init: Partial<KeyboardEvent> & { key: string; altGraph?: boolean }): KeyboardEvent {
  return {
    key: init.key,
    code: init.code ?? '',
    metaKey: init.metaKey ?? false,
    ctrlKey: init.ctrlKey ?? false,
    shiftKey: init.shiftKey ?? false,
    altKey: init.altKey ?? false,
    getModifierState: (name: string) => name === 'AltGraph' && init.altGraph === true
  } as unknown as KeyboardEvent;
}

describe('chordSpec', () => {
  it.each<[string, Partial<KeyboardEvent> & { key: string; altGraph?: boolean }, string | null]>([
    ['a letter', { key: 'w', code: 'KeyW' }, 'w'],
    ['Shift with a letter', { key: 'W', code: 'KeyW', shiftKey: true }, 'Shift+w'],
    ['Cmd with Shift', { key: 'u', code: 'KeyU', metaKey: true, shiftKey: true }, 'Mod+Shift+u'],
    ['Ctrl', { key: 'k', code: 'KeyK', ctrlKey: true }, 'Mod+k'],
    ['Mac Option letter', { key: 'å', code: 'KeyA', altKey: true }, 'Alt+a'],
    [
      'all modifiers',
      { key: 'a', code: 'KeyA', metaKey: true, altKey: true, shiftKey: true },
      'Mod+Alt+Shift+a'
    ],
    ['a plain digit', { key: '3', code: 'Digit3' }, '3'],
    ['Shift with a digit', { key: '#', code: 'Digit3', shiftKey: true }, 'Shift+Digit3'],
    ['Alt with a digit', { key: '¡', code: 'Digit1', altKey: true }, 'Alt+Digit1'],
    ['Space', { key: ' ', code: 'Space' }, 'Space'],
    ['Shift with a symbol', { key: '?', code: 'Slash', shiftKey: true }, '?'],
    ['AltGr symbol', { key: '[', ctrlKey: true, altKey: true, altGraph: true }, '['],
    ['Cmd with a symbol', { key: "'", code: 'Quote', metaKey: true }, "Mod+'"],
    ['a named key', { key: 'Enter', code: 'Enter', shiftKey: true }, 'Shift+Enter'],
    ['a function key', { key: 'F2', code: 'F2' }, 'F2'],
    ['a Norwegian letter', { key: 'ø', code: 'Semicolon' }, null],
    ['a modifier alone', { key: 'Shift', code: 'ShiftLeft', shiftKey: true }, null],
    ['Escape', { key: 'Escape', code: 'Escape' }, null],
    ['a dead key', { key: 'Dead', code: 'BracketRight' }, null]
  ])('records %s', (_, init, spec) => {
    expect(chordSpec(press(init))).toBe(spec);
  });

  it.each<Partial<KeyboardEvent> & { key: string }>([
    { key: 'W', code: 'KeyW', shiftKey: true },
    { key: 'å', code: 'KeyA', altKey: true },
    { key: '#', code: 'Digit3', shiftKey: true },
    { key: '?', code: 'Slash', shiftKey: true },
    { key: 'u', code: 'KeyU', metaKey: true, shiftKey: true }
  ])('matches what it records for %o', (init) => {
    const event = press(init);
    expect(chordMatches(event, parseChord(chordSpec(event) ?? ''))).toBe(true);
  });
});
