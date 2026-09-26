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
    code: init.code ?? '',
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
  'geometry',
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
    ['paneSwap', { key: 'ArrowLeft', altKey: true }, true],
    ['paneSwap', { key: 'ArrowLeft', shiftKey: true }, false],
    ['paneSwap', { key: 'ArrowLeft' }, false],
    ['autoAdjust', { key: 'u', metaKey: true }, true],
    ['autoAdjust', { key: 'u' }, false],
    ['unflag', { key: 'u', metaKey: true }, false]
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
    [{ key: 'f', shiftKey: true }, ['loupe', 'global'], 'fullscreen'],
    [{ key: 'f', shiftKey: true }, ['compare', 'global'], 'fullscreen'],
    [{ key: 'f', shiftKey: true }, ['survey', 'global'], 'fullscreen'],
    [{ key: 'c' }, ['retouch', 'editor', 'global'], 'retouchClone'],
    [{ key: 'Enter' }, ['geometry', 'editor', 'global'], 'geometryDone'],
    [{ key: 'Enter' }, ['editor', 'global'], null],
    [{ key: 'Escape' }, ['geometry', 'editor', 'global'], 'editorEscape'],
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

  it('gives every keyless bind a display and no match', () => {
    const keyless = all.filter((bind) => bind.keys.length === 0);
    expect(keyless.length).toBeGreaterThan(0);
    for (const bind of keyless) {
      expect(bind.display, bind.id).toBeTruthy();
      expect(matchKeybind(key({ key: 'Shift' }), bind.contexts)).not.toBe(bind.id);
    }
  });
});

describe('code chords', () => {
  it.each<[string, Partial<KeyboardEvent> & { key: string }, KeybindId | null]>([
    ['Norwegian Shift+0', { key: '=', code: 'Digit0', shiftKey: true }, 'rateAdvance'],
    ['US Shift+3', { key: '#', code: 'Digit3', shiftKey: true }, 'rateAdvance'],
    ['plain 3', { key: '3', code: 'Digit3' }, 'rate'],
    ['Norwegian = without Shift', { key: '=', code: 'Minus' }, 'gridSize'],
    ['Shift+6', { key: '&', code: 'Digit6', shiftKey: true }, null]
  ])('%s', (_name, init, expected) => {
    expect(matchKeybind(key(init), ['grid'])).toBe(expected);
  });

  it('labels a code chord by its digit', () => {
    expect(formatChord('Shift+Digit3', false)).toBe('Shift+3');
  });
});
