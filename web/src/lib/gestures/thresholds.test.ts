import { describe, expect, it } from 'vitest';
import { DEFAULT_THRESHOLDS, thresholds } from '#lib/gestures/thresholds.ts';

describe('thresholds', () => {
  it('applies overrides on top of the defaults without touching them', () => {
    expect(thresholds({ panSlop: 4 })).toEqual({ ...DEFAULT_THRESHOLDS, panSlop: 4 });
    expect(DEFAULT_THRESHOLDS.panSlop).not.toBe(4);
  });
});
