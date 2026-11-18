import { describe, expect, it } from 'vitest';
import { sortFields, sortLabel } from './sortOptions';

describe('sortFields', () => {
  it('offers rating and size only with structured search', () => {
    expect(sortFields(false)).toEqual(['taken', 'added']);
    expect(sortFields(true)).toEqual(['taken', 'added', 'rating', 'size']);
  });
});

describe('sortLabel', () => {
  it('names the field and the direction', () => {
    expect(sortLabel('taken', 'asc')).toBe('Sort: date taken, oldest first');
    expect(sortLabel('rating', 'desc')).toBe('Sort: rating, highest first');
  });
});
