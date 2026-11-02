import { describe, expect, it } from 'vitest';
import { hexToOklch } from './color';
import { accentRamp, neutralLadder, RAMP_STEPS } from './ramps';
import type { Palette } from './types';

const VIOLET_DARK = [
  '#221533',
  '#2f1d46',
  '#3c2559',
  '#4d306f',
  '#7046a0',
  '#b7a1fb',
  '#c4b5fd',
  '#d0c5fd',
  '#ddd6fe',
  '#ede9fe',
  '#f5f3ff'
];

describe('ramps', () => {
  it('derives a dark violet ramp close to the hand-tuned one', () => {
    const derived = accentRamp('#b7a1fb', '#000000', '#fafafa', 'dark');
    expect(derived).toHaveLength(RAMP_STEPS.length);
    expect(derived[5]).toBe('#b7a1fb');
    derived.forEach((hex, index) => {
      expect(Math.abs(hexToOklch(hex).l - hexToOklch(VIOLET_DARK[index] ?? '').l)).toBeLessThan(
        0.02
      );
    });
  });

  it('runs from background to foreground in both schemes', () => {
    for (const [scheme, base, text] of [
      ['dark', '#1e1e2e', '#cdd6f4'],
      ['light', '#ffffff', '#1c171f']
    ] as const) {
      const lightness = accentRamp('#89b4fa', base, text, scheme).map((hex) => hexToOklch(hex).l);
      const ordered = scheme === 'dark' ? [...lightness] : [...lightness].reverse();
      expect(ordered).toEqual([...ordered].sort((a, b) => a - b));
    }
  });

  it('builds the neutral ladder through every surface slot', () => {
    const palette: Palette = {
      crust: '#11111b',
      mantle: '#181825',
      base: '#1e1e2e',
      surface0: '#313244',
      surface1: '#45475a',
      surface2: '#585b70',
      overlay: '#7f849c',
      subtext: '#a6adc8',
      text: '#cdd6f4',
      success: '#a6e3a1',
      warning: '#f9e2af',
      danger: '#f38ba8',
      info: '#89b4fa'
    };
    const ladder = neutralLadder(palette);
    expect(ladder).toHaveLength(11);
    expect([ladder[0], ladder[1], ladder[2], ladder[3], ladder[5], ladder[6], ladder[10]]).toEqual([
      '#1e1e2e',
      '#313244',
      '#45475a',
      '#585b70',
      '#7f849c',
      '#a6adc8',
      '#cdd6f4'
    ]);
  });
});
