import { describe, expect, it } from 'vitest';
import { contrastRatio, hexToOklch, mix, oklchToHex } from './color';

describe('color', () => {
  it('round-trips hex through OKLCH', () => {
    for (const hex of ['#000000', '#ffffff', '#b7a1fb', '#1e1e2e', '#f38ba8', '#007f79']) {
      expect(oklchToHex(hexToOklch(hex))).toBe(hex);
    }
  });

  it('reads known OKLCH values', () => {
    const violet = hexToOklch('#b7a1fb');
    expect(violet.l).toBeCloseTo(0.762, 3);
    expect(violet.c).toBeCloseTo(0.128, 3);
    expect(violet.h).toBeCloseTo(294.4, 1);
    expect(hexToOklch('#808080').c).toBeLessThan(1e-6);
  });

  it('pulls out-of-gamut colours back by chroma', () => {
    const hex = oklchToHex({ l: 0.7, c: 0.4, h: 190 });
    expect(hex).toMatch(/^#[0-9a-f]{6}$/);
    expect(hexToOklch(hex).h).toBeCloseTo(190, 0);
  });

  it('mixes in OKLab between both ends', () => {
    expect(mix('#000000', '#ffffff', 0)).toBe('#000000');
    expect(mix('#000000', '#ffffff', 1)).toBe('#ffffff');
    expect(hexToOklch(mix('#000000', '#ffffff', 0.5)).l).toBeCloseTo(0.5, 2);
  });

  it('computes WCAG contrast', () => {
    expect(contrastRatio('#000000', '#ffffff')).toBeCloseTo(21, 6);
    expect(contrastRatio('#777777', '#ffffff')).toBeCloseTo(4.48, 2);
  });

  it('rejects anything but #rrggbb', () => {
    expect(() => hexToOklch('#abc')).toThrow('not a #rrggbb colour: #abc');
    expect(() => hexToOklch('red')).toThrow('not a #rrggbb colour: red');
  });
});
