import { DoubleTap } from '#lib/map/doubleTap.ts';
import { describe, expect, it } from 'vitest';

describe('double tap', () => {
  it('pairs two taps inside the window and starts over after one', () => {
    const taps = new DoubleTap(300);
    expect(taps.tap(0)).toBe('single');
    expect(taps.tap(250)).toBe('double');
    expect(taps.tap(700)).toBe('single');
    expect(taps.tap(1200)).toBe('single');
  });

  it('trusts a click count of two from the browser', () => {
    expect(new DoubleTap(300).tap(0, 2)).toBe('double');
  });

  it('ignores extra clicks right after a double so a mark is not undone', () => {
    const taps = new DoubleTap(300);
    expect(taps.tap(0, 1)).toBe('single');
    expect(taps.tap(120, 2)).toBe('double');
    expect(taps.tap(180, 2)).toBe('ignore');
    expect(taps.tap(260, 3)).toBe('ignore');
    expect(taps.tap(900, 1)).toBe('single');
  });
});
