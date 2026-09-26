import { describe, expect, it } from 'vitest';
import { commonValue, coverage } from './selectionState';

describe('selection state', () => {
  it.each([
    { flags: [true, true], selected: 2, want: 'all' },
    { flags: [false, false], selected: 2, want: 'none' },
    { flags: [true, false], selected: 2, want: 'some' },
    { flags: [false], selected: 2, want: 'some' },
    { flags: [true], selected: 2, want: 'some' }
  ])('coverage of $flags out of $selected is $want', ({ flags, selected, want }) => {
    expect(coverage(flags, selected, (f) => f)).toBe(want);
  });

  it.each([
    { ratings: [3, 3], selected: 2, want: 3 },
    { ratings: [3, 0], selected: 2, want: null },
    { ratings: [3], selected: 2, want: null },
    { ratings: [], selected: 0, want: null }
  ])('common value of $ratings out of $selected is $want', ({ ratings, selected, want }) => {
    expect(commonValue(ratings, selected, (r) => r)).toBe(want);
  });
});
