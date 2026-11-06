import { afterEach, describe, expect, it } from 'vitest';
import {
  clear,
  held,
  heldAlone,
  keyHeld,
  latch,
  latched,
  plainClick,
  type ModifierKeys,
  type ModifierRole
} from './modifiers.svelte.ts';

const NONE: ModifierKeys = { shiftKey: false, altKey: false, ctrlKey: false, metaKey: false };

function keys(over: Partial<ModifierKeys>): ModifierKeys {
  return { ...NONE, ...over };
}

type Key = 'shift' | 'alt' | 'mod';

const TABLE: Record<ModifierRole, Key> = {
  constrain: 'shift',
  straighten: 'shift',
  fromCentre: 'alt',
  pan: 'mod',
  zoom: 'mod',
  coarse: 'shift',
  exclude: 'shift',
  sample: 'alt',
  extend: 'shift',
  toggle: 'mod',
  solo: 'alt',
  preview: 'alt',
  resetAll: 'shift',
  resize: 'alt',
  hardness: 'shift',
  lineBreak: 'shift'
};

const PRESSES: [Key, ModifierKeys][] = [
  ['shift', keys({ shiftKey: true })],
  ['alt', keys({ altKey: true })],
  ['mod', keys({ ctrlKey: true })],
  ['mod', keys({ metaKey: true })]
];

afterEach(() => clear());

describe('held', () => {
  it.each(Object.entries(TABLE) as [ModifierRole, Key][])('maps %s to %s only', (role, key) => {
    expect(held(role, NONE)).toBe(false);
    for (const [pressed, e] of PRESSES) {
      expect(held(role, e)).toBe(pressed === key);
    }
  });

  it('gives the same key different roles', () => {
    const e = keys({ shiftKey: true });
    expect(held('constrain', e)).toBe(true);
    expect(held('exclude', e)).toBe(true);
    expect(held('sample', e)).toBe(false);
  });

  it('counts a latched toggle as held without any key', () => {
    latch('exclude', true);
    expect(held('exclude', NONE)).toBe(true);
    expect(held('constrain', NONE)).toBe(false);
  });
});

describe('keyHeld', () => {
  it('reads only the keyboard and ignores latches', () => {
    latch('extend', true);
    expect(keyHeld('extend', NONE)).toBe(false);
    expect(keyHeld('extend', keys({ shiftKey: true }))).toBe(true);
  });
});

describe('heldAlone', () => {
  it.each([
    [keys({ shiftKey: true }), true],
    [keys({ shiftKey: true, altKey: true }), false],
    [keys({ shiftKey: true, ctrlKey: true }), false],
    [keys({ shiftKey: true, metaKey: true }), false],
    [keys({ altKey: true }), false],
    [NONE, false]
  ])('reads coarse for %o as %s', (e, expected) => {
    expect(heldAlone('coarse', e)).toBe(expected);
  });

  it('accepts a latch while other keys stay up', () => {
    latch('coarse', true);
    expect(heldAlone('coarse', NONE)).toBe(true);
    expect(heldAlone('coarse', keys({ altKey: true }))).toBe(false);
  });
});

describe('latch', () => {
  it('sets, unsets and clears roles', () => {
    latch('sample', true);
    latch('sample', true);
    latch('constrain', true);
    expect(latched('sample')).toBe(true);
    latch('sample', false);
    expect(latched('sample')).toBe(false);
    expect(latched('constrain')).toBe(true);
    clear();
    expect(latched('constrain')).toBe(false);
  });
});

describe('plainClick', () => {
  it('accepts an unmodified primary click', () => {
    expect(plainClick({ ...NONE, button: 0 })).toBe(true);
  });

  it.each([
    ['a middle click', { button: 1 }],
    ['shift', { button: 0, shiftKey: true }],
    ['alt', { button: 0, altKey: true }],
    ['ctrl', { button: 0, ctrlKey: true }],
    ['meta', { button: 0, metaKey: true }]
  ])('leaves %s to the browser', (_, over) => {
    expect(plainClick({ ...NONE, ...over })).toBe(false);
  });
});
