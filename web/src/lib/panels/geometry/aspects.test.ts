import { describe, expect, it } from 'vitest';
import {
  aspectKey,
  aspectText,
  isPresetRatio,
  orientedAspect,
  parseRatio,
  parseRatioSide,
  ratioLabel,
  ratioReadout,
  sourceRatioLabel
} from './aspects';

describe('crop aspects', () => {
  it('labels a ratio in the requested orientation', () => {
    expect(ratioLabel({ num: 5, den: 4 })).toBe('5:4');
    expect(ratioLabel({ num: 4, den: 5 })).toBe('5:4');
    expect(ratioLabel({ num: 5, den: 4 }, true)).toBe('4:5');
  });

  it('keys a ratio the same in both orientations', () => {
    expect(aspectKey({ kind: 'ratio', num: 4, den: 5 })).toBe('r-5-4');
    expect(aspectKey({ kind: 'ratio', num: 5, den: 4 })).toBe('r-5-4');
    expect(aspectKey({ kind: 'free' })).toBe('free');
  });

  it('orients a preset to the current orientation', () => {
    expect(orientedAspect({ num: 5, den: 4 }, true)).toEqual({ kind: 'ratio', num: 4, den: 5 });
    expect(orientedAspect({ num: 4, den: 5 }, false)).toEqual({ kind: 'ratio', num: 5, den: 4 });
  });

  it('knows the presets', () => {
    expect(isPresetRatio({ num: 2, den: 3 })).toBe(true);
    expect(isPresetRatio({ num: 21, den: 9 })).toBe(false);
  });

  it('shows typed text and a decimal readout', () => {
    expect(aspectText({ kind: 'ratio', num: 239, den: 100 })).toBe('2.39:1');
    expect(aspectText({ kind: 'ratio', num: 100, den: 239 })).toBe('1:2.39');
    expect(aspectText({ kind: 'ratio', num: 21, den: 9 })).toBe('21:9');
    expect(ratioLabel({ num: 239, den: 100 }, true)).toBe('1:2.39');
    expect(ratioLabel({ num: 10, den: 1 })).toBe('10:1');
    expect(aspectText({ kind: 'original' })).toBe('');
    expect(ratioReadout({ num: 239, den: 100 })).toBe('2.39 : 1');
    expect(ratioReadout({ num: 9, den: 16 })).toBe('1.78 : 1');
    expect(ratioReadout({ num: 1, den: 1 })).toBe('1 : 1');
  });

  it.each([
    ['21:9', { num: 21, den: 9 }],
    [' 65 : 24 ', { num: 65, den: 24 }],
    ['9x16', { num: 9, den: 16 }],
    ['4/5', { num: 4, den: 5 }],
    ['2.39', { num: 239, den: 100 }],
    ['1.5', { num: 3, den: 2 }],
    ['2.35', { num: 235, den: 100 }],
    ['1:2.39', { num: 100, den: 239 }],
    ['2.39:1', { num: 239, den: 100 }],
    ['3', { num: 3, den: 1 }],
    ['0:1', null],
    ['1.2345', null],
    ['10000:1', null],
    ['a:b', null],
    ['', null],
    ['-2:1', null]
  ])('parses %j as %o', (text, want) => {
    expect(parseRatio(text)).toEqual(want);
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

  it('labels the source shape as reduced sides or a decimal', () => {
    expect(sourceRatioLabel(6000, 4000)).toBe('3:2');
    expect(sourceRatioLabel(4000, 6000)).toBe('2:3');
    expect(sourceRatioLabel(5472, 3648)).toBe('3:2');
    expect(sourceRatioLabel(6024, 4024)).toBe('1.5 : 1');
  });
});
