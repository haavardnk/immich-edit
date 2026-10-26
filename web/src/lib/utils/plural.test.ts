import { describe, expect, it } from 'vitest';
import { countOf } from './plural';

describe('countOf', () => {
  it('picks the singular only for one', () => {
    expect(countOf(1, 'photo')).toBe('1 photo');
    expect(countOf(0, 'photo')).toBe('0 photos');
    expect(countOf(2, 'virtual copy', 'virtual copies')).toBe('2 virtual copies');
  });
});
