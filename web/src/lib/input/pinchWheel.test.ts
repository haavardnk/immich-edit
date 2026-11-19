import { describe, expect, it } from 'vitest';
import { isPinchWheel } from './pinchWheel';

describe('pinch wheel', () => {
  it('tells a Ctrl trackpad pinch stream from mouse notches and Cmd scrolling', () => {
    const sample = { ctrlKey: true, metaKey: false, deltaMode: 0, deltaY: -4 };
    expect(isPinchWheel(sample)).toBe(true);
    expect(isPinchWheel({ ...sample, metaKey: true })).toBe(false);
    expect(isPinchWheel({ ...sample, ctrlKey: false, metaKey: true })).toBe(false);
    expect(isPinchWheel({ ...sample, deltaMode: 1, deltaY: -3 })).toBe(false);
    expect(isPinchWheel({ ...sample, deltaY: 100 })).toBe(false);
  });
});
