import { centreShift } from '#lib/map/resizeHold.ts';
import { describe, expect, it } from 'vitest';

describe('resize hold', () => {
  it('measures how far the centre moved on screen', () => {
    const before = { left: 0, top: 100, width: 400, height: 600 };
    expect(centreShift(before, { ...before, height: 680 })).toEqual({ x: 0, y: 40 });
    expect(centreShift(before, { ...before, top: 120, height: 580 })).toEqual({ x: 0, y: 10 });
    expect(centreShift(before, before)).toEqual({ x: 0, y: 0 });
  });
});
