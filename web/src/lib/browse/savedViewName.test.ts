import { describe, expect, it } from 'vitest';
import { SAVED_VIEW_NAME_MAX, savedViewName, savedViewNameError } from './savedViewName';

describe('savedViewName', () => {
  it('joins the active filter values', () => {
    expect(savedViewName(['Norway', 'X-T5'], 'Photos')).toBe('Norway · X-T5');
  });

  it('falls back to the page title without filters', () => {
    expect(savedViewName([], 'Favorites')).toBe('Favorites');
  });

  it('cuts long names to the limit by characters', () => {
    const name = savedViewName(['ø'.repeat(70), 'å'.repeat(30)], 'Photos');
    expect([...name]).toHaveLength(SAVED_VIEW_NAME_MAX);
  });

  it('does not end a cut name in a space', () => {
    expect(savedViewName(['a'.repeat(79), 'b'], 'Photos')).toBe('a'.repeat(79));
  });
});

describe('savedViewNameError', () => {
  it.each([
    ['Norway', null],
    ['  Norway  ', null],
    ['   ', 'Enter a name'],
    ['', 'Enter a name'],
    ['x'.repeat(80), null],
    ['x'.repeat(81), 'Up to 80 characters'],
    ['ø'.repeat(80), null]
  ])('judges %j as %s', (name, error) => {
    expect(savedViewNameError(name)).toBe(error);
  });
});
