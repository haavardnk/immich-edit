import { describe, expect, it } from 'vitest';
import { isKeybind, matchKeybind } from './match';
import type { KeybindContext, KeybindId } from './table';

function key(init: Partial<KeyboardEvent> & { key: string; altGraph?: boolean }): KeyboardEvent {
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
    ['favorite', { key: 'p' }, true],
    ['favorite', { key: 'f' }, false],
    ['fullscreen', { key: 'f' }, true],
    ['fullscreen', { key: 'F', shiftKey: true }, false],
    ['copyEdits', { key: 'c', metaKey: true }, true],
    ['copyEdits', { key: 'c', metaKey: true, shiftKey: true }, false],
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
    [{ key: 'd' }, ['editor', 'global'], null],
    [{ key: 'D', shiftKey: true }, ['editor', 'global'], 'openDevelop'],
    [{ key: 'r' }, ['geometry', 'editor', 'global'], 'addRadialLayer'],
    [{ key: 'w' }, ['masks', 'editor', 'global'], 'pickWhiteBalance'],
    [{ key: 'S', shiftKey: true }, ['editor', 'global'], 'filmstrip'],
    [{ key: 'i' }, ['grid', 'global'], 'gridTileInfo'],
    [{ key: 'i' }, ['loupe', 'global'], 'toggleInfo'],
    [{ key: 'f' }, ['loupe', 'global'], 'fullscreen'],
    [{ key: 'f' }, ['compare', 'global'], 'fullscreen'],
    [{ key: 'f' }, ['survey', 'global'], 'fullscreen'],
    [{ key: 'F', shiftKey: true }, ['grid', 'global'], 'toggleFilters'],
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
});

describe('layout symbols', () => {
  it.each<[string, Partial<KeyboardEvent> & { key: string; altGraph?: boolean }, string, boolean]>([
    ['Norwegian Mac [', { key: '[', altKey: true }, 'brushSize', true],
    ['Norwegian Mac {', { key: '{', altKey: true, shiftKey: true }, 'brushHardness', true],
    ['Norwegian Mac \\', { key: '\\', altKey: true, shiftKey: true }, 'holdOriginal', true],
    [
      'Windows AltGr [',
      { key: '[', ctrlKey: true, altKey: true, altGraph: true },
      'brushSize',
      true
    ],
    ['ISO key beside Z', { key: '<' }, 'holdOriginal', true],
    ['Ctrl+[ without AltGr', { key: '[', ctrlKey: true }, 'brushSize', false],
    ['Cmd+[', { key: '[', metaKey: true, altKey: true }, 'brushSize', false],
    [
      'AltGr does not unlock a letter',
      { key: 'z', ctrlKey: true, altKey: true, altGraph: true },
      'undo',
      false
    ]
  ])('%s', (_name, init, id, expected) => {
    expect(isKeybind(key(init), id as KeybindId)).toBe(expected);
  });
});
