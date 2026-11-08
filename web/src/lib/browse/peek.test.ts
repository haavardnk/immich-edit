import { describe, expect, it } from 'vitest';
import type { CullState } from './cullState';
import { peek, peekMarks } from './peek.svelte';

const PLAIN: CullState = {
  rating: 0,
  isFavorite: false,
  rejected: false,
  label: null,
  owned: true
};

describe('peek', () => {
  it('tracks the hovered tile until the pointer leaves it', () => {
    peek.hover('a');
    peek.hover('b');
    peek.leave('a');
    expect(peek.hovered).toBe('b');
    peek.leave('b');
    expect(peek.hovered).toBeNull();
  });

  it('shows and hides a photo', () => {
    peek.show('a');
    expect(peek.id).toBe('a');
    peek.hide();
    expect(peek.id).toBeNull();
  });
});

describe('peekMarks', () => {
  it('names nothing for a plain photo', () => {
    expect(peekMarks(PLAIN, false)).toEqual([]);
  });

  it('names the flags, label and edits in order', () => {
    expect(peekMarks({ ...PLAIN, isFavorite: true, rejected: true, label: 'red' }, true)).toEqual([
      'Favorite',
      'Rejected',
      'Red label',
      'Edited'
    ]);
  });
});
