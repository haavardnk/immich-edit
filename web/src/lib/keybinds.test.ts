import { describe, expect, it } from 'vitest';
import {
  KEYBINDS,
  formatChord,
  isKeybind,
  keyLabel,
  keysFor,
  matchKeybind,
  type Keybind,
  type KeybindContext,
  type KeybindId
} from './keybinds';

function key(init: Partial<KeyboardEvent> & { key: string }): KeyboardEvent {
  return {
    key: init.key,
    metaKey: init.metaKey ?? false,
    ctrlKey: init.ctrlKey ?? false,
    shiftKey: init.shiftKey ?? false,
    altKey: init.altKey ?? false
  } as KeyboardEvent;
}

const CONTEXTS: KeybindContext[] = [
  'global',
  'grid',
  'loupe',
  'compare',
  'survey',
  'editor',
  'masks',
  'retouch'
];

describe('chord matching', () => {
  it.each<[string, Partial<KeyboardEvent> & { key: string }, boolean]>([
    ['undo', { key: 'z', metaKey: true }, true],
    ['undo', { key: 'z', ctrlKey: true }, true],
    ['undo', { key: 'z' }, false],
    ['undo', { key: 'z', metaKey: true, shiftKey: true }, false],
    ['redo', { key: 'z', metaKey: true, shiftKey: true }, true],
    ['redo', { key: 'z', metaKey: true }, false],
    ['undo', { key: 'z', metaKey: true, altKey: true }, false],
    ['favorite', { key: 'P', shiftKey: true }, false],
    ['favorite', { key: 'P' }, true],
    ['favorite', { key: 'p' }, true],
    ['fullscreen', { key: 'f', shiftKey: true }, true],
    ['fullscreen', { key: 'f' }, false],
    ['help', { key: '?', shiftKey: true }, true],
    ['help', { key: '/', shiftKey: true }, true],
    ['zoomToggle', { key: ' ' }, true],
    ['togglePanels', { key: 'Tab' }, true],
    ['toggleChrome', { key: 'Tab', shiftKey: true }, true],
    ['togglePanels', { key: 'Tab', shiftKey: true }, false],
    ['brushSize', { key: '[' }, true],
    ['brushHardness', { key: '{' }, true],
    ['gridSize', { key: '+' }, true],
    ['gridSize', { key: '=' }, true],
    ['paneSwap', { key: 'ArrowLeft', shiftKey: true }, true],
    ['paneSwap', { key: 'ArrowLeft' }, false]
  ])('%s matches %o -> %s', (id, event, expected) => {
    expect(isKeybind(key(event), id as KeybindId)).toBe(expected);
  });
});

describe('context resolution', () => {
  it.each<[Partial<KeyboardEvent> & { key: string }, KeybindContext[], KeybindId | null]>([
    [{ key: 'Enter' }, ['grid', 'global'], 'openEditor'],
    [{ key: 'Enter' }, ['compare', 'global'], 'panePromote'],
    [{ key: 'Enter' }, ['survey', 'global'], 'surveyKeep'],
    [{ key: 'Escape' }, ['editor', 'global'], 'editorEscape'],
    [{ key: 'Escape' }, ['masks', 'editor', 'global'], 'editorEscape'],
    [{ key: 'Escape' }, ['grid', 'global'], 'gridClearSelection'],
    [{ key: 'd' }, ['compare', 'global'], 'paneOpenEditor'],
    [{ key: 'd' }, ['editor', 'global'], 'openDevelop'],
    [{ key: 'c' }, ['retouch', 'editor', 'global'], 'retouchClone'],
    [{ key: 'c' }, ['grid', 'global'], 'enterCompare'],
    [{ key: 'q' }, ['grid', 'global'], null]
  ])('%o in %o resolves to %s', (event, contexts, expected) => {
    expect(matchKeybind(key(event), contexts)).toBe(expected);
  });
});

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

  it('formats bare chords', () => {
    expect(formatChord('Mod+Shift+e', true)).toBe('⌘⇧E');
    expect(formatChord('Mod+Shift+e', false)).toBe('Ctrl+Shift+E');
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
});
