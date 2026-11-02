import { describe, expect, it } from 'vitest';
import { contrastRatio, hexToOklch } from './color';
import { backgroundColor, CONTRAST_PAIRS, gate, nudge, pairContrast } from './contrast';
import type { Palette } from './types';

const PALETTE: Palette = {
  crust: '#dce0e8',
  mantle: '#e6e9ef',
  base: '#eff1f5',
  surface0: '#ccd0da',
  surface1: '#bcc0cc',
  surface2: '#acb0be',
  overlay: '#8c8fa1',
  subtext: '#5c5f77',
  text: '#4c4f69',
  success: '#40a02b',
  warning: '#df8e1d',
  danger: '#d20f39',
  info: '#1e66f5'
};

describe('contrast gate', () => {
  it('leaves a passing pair alone', () => {
    expect(nudge('#000000', '#ffffff', 4.5)).toBe('#000000');
  });

  it('moves lightness away from the background until the pair passes', () => {
    const before = hexToOklch('#df8e1d');
    const after = nudge('#df8e1d', '#eff1f5', 4.5);
    expect(contrastRatio(after, '#eff1f5')).toBeGreaterThanOrEqual(4.5);
    expect(contrastRatio(nudge('#df8e1d', '#eff1f5', 4.4), '#eff1f5')).toBeLessThan(4.6);
    const moved = hexToOklch(after);
    expect(moved.l).toBeLessThan(before.l);
    expect(moved.h).toBeCloseTo(before.h, 0);
  });

  it('lightens on a dark background', () => {
    expect(hexToOklch(nudge('#444444', '#000000', 4.5)).l).toBeGreaterThan(hexToOklch('#444444').l);
  });

  it('records each nudged pair and passes every pair afterwards', () => {
    const result = gate(PALETTE, '#df8e1d');
    expect(result.nudges).toEqual([
      'text/surface1',
      'text@65/crust',
      'subtext/surface0',
      'accent/base',
      'accent/mantle',
      'accent/crust',
      'accent/accentTint',
      'success/base',
      'success/mantle',
      'warning/base',
      'warning/mantle',
      'danger/mantle',
      'info/base',
      'info/mantle',
      'overlay/base'
    ]);
    const colors = { ...result.palette, accent: result.accent };
    for (const { foreground, background, minimum, alpha } of CONTRAST_PAIRS) {
      expect(
        pairContrast(colors[foreground], backgroundColor(colors, background), alpha)
      ).toBeGreaterThanOrEqual(minimum);
    }
  });
});
