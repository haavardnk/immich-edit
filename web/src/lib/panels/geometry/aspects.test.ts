import { describe, expect, it } from 'vitest';
import {
  aspectForKey,
  aspectOptions,
  CUSTOM_KEY,
  parseRatioSide,
  selectedAspectKey
} from './aspects';

describe('crop aspects', () => {
  it('labels presets in the current orientation', () => {
    const labels = (portrait: boolean) => aspectOptions(portrait).map((o) => o.label);
    expect(labels(false)).toContain('5:4');
    expect(labels(true)).toEqual(expect.arrayContaining(['4:5', '5:7', '2:3']));
  });

  it.each([
    [{ kind: 'ratio', num: 4, den: 5 } as const, 'r-5-4'],
    [{ kind: 'ratio', num: 7, den: 5 } as const, 'r-7-5'],
    [{ kind: 'ratio', num: 7, den: 3 } as const, CUSTOM_KEY],
    [{ kind: 'free' } as const, 'free']
  ])('selects %o as %s', (aspect, key) => {
    expect(selectedAspectKey(aspect)).toBe(key);
  });

  it('keeps the orientation when switching presets', () => {
    expect(aspectForKey('r-5-4', true)).toEqual({ kind: 'ratio', num: 4, den: 5 });
    expect(aspectForKey('r-5-4', false)).toEqual({ kind: 'ratio', num: 5, den: 4 });
    expect(aspectForKey(CUSTOM_KEY, false)).toBeNull();
  });

  it.each([
    ['7', 7],
    [' 12 ', 12],
    ['0', null],
    ['1.5', null],
    ['', null],
    ['10000', null]
  ])('parses ratio side %j as %s', (raw, want) => {
    expect(parseRatioSide(raw)).toBe(want);
  });
});
