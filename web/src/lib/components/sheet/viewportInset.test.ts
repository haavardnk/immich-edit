import { describe, expect, it } from 'vitest';
import { keyboardInset } from '#lib/components/sheet/viewportInset.ts';

describe('keyboardInset', () => {
  it('is empty when the visual viewport fills the window', () => {
    expect(keyboardInset({ innerHeight: 800, height: 800, offsetTop: 0 })).toEqual({
      top: 0,
      bottom: 0
    });
  });

  it('reserves the keyboard height at the bottom', () => {
    expect(keyboardInset({ innerHeight: 800, height: 500, offsetTop: 0 })).toEqual({
      top: 0,
      bottom: 300
    });
  });

  it('follows a visual viewport scrolled down by the browser', () => {
    expect(keyboardInset({ innerHeight: 800, height: 500, offsetTop: 120 })).toEqual({
      top: 120,
      bottom: 180
    });
  });

  it('rounds fractional metrics and never goes negative', () => {
    expect(keyboardInset({ innerHeight: 800, height: 820.4, offsetTop: -3 })).toEqual({
      top: 0,
      bottom: 0
    });
    expect(keyboardInset({ innerHeight: 800, height: 499.6, offsetTop: 0.2 })).toEqual({
      top: 0,
      bottom: 300
    });
  });
});
