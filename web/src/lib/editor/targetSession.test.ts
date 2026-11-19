import { PERCENT } from '#lib/editor/params/range.ts';
import { describe, expect, it } from 'vitest';
import { TARGET_DRAG_PX, draggedValue } from './targetSession';

describe('draggedValue', () => {
  it('raises the value when dragging up and lowers it when dragging down', () => {
    expect(draggedValue(PERCENT, 0, -TARGET_DRAG_PX / 4)).toBe(50);
    expect(draggedValue(PERCENT, 10, TARGET_DRAG_PX / 2)).toBe(-90);
  });

  it('clamps to the range', () => {
    expect(draggedValue(PERCENT, 80, -TARGET_DRAG_PX)).toBe(100);
    expect(draggedValue(PERCENT, -80, TARGET_DRAG_PX)).toBe(-100);
  });
});
