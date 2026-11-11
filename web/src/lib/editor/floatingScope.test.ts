import { describe, expect, it } from 'vitest';
import { nearestCorner, yieldsTo } from './floatingScope';

const BOX = { w: 800, h: 600 };

describe('nearestCorner', () => {
  it.each([
    [{ x: 10, y: 10 }, 'tl'],
    [{ x: 790, y: 20 }, 'tr'],
    [{ x: 30, y: 580 }, 'bl'],
    [{ x: 700, y: 500 }, 'br'],
    [{ x: 400, y: 300 }, 'br'],
    [{ x: -50, y: 900 }, 'bl']
  ] as const)('snaps a centre at %o to %s', (centre, corner) => {
    expect(nearestCorner(centre, BOX)).toBe(corner);
  });
});

describe('yieldsTo', () => {
  it('stays for Light sliders and when nothing drags', () => {
    expect(yieldsTo(null)).toBe(false);
    expect(yieldsTo('exposure')).toBe(false);
    expect(yieldsTo('shadows')).toBe(false);
  });

  it('hides for sliders outside Light', () => {
    expect(yieldsTo('saturation')).toBe(true);
    expect(yieldsTo('')).toBe(true);
  });
});
