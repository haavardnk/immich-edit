import { tagMenu } from '#lib/browse/tagActions.ts';
import { describe, expect, it } from 'vitest';

describe('tagMenu', () => {
  it('offers Open tag only in lists', () => {
    expect(tagMenu('list')).toEqual([['open'], ['rename', 'color', 'subtag'], ['delete']]);
    expect(tagMenu('page')).toEqual([['rename', 'color', 'subtag'], ['delete']]);
  });
});
