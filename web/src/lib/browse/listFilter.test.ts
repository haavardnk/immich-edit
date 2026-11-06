import { describe, expect, it } from 'vitest';
import { LIST_FILTER_MIN, filterByName, listFilterShown, listFilterStatus } from './listFilter';

const TAGS = ['landscape', 'Iceland', 'lanterns', 'Finland', 'Bergen', 'Ålesund'];
const NOUN = { one: 'tag', many: 'tags' };

describe('long list filter', () => {
  it('appears only above the threshold', () => {
    expect(listFilterShown(LIST_FILTER_MIN)).toBe(false);
    expect(listFilterShown(LIST_FILTER_MIN + 1)).toBe(true);
  });

  it('keeps matching items in their order', () => {
    expect(filterByName(TAGS, 'lan', (tag) => tag)).toEqual([
      'landscape',
      'Iceland',
      'lanterns',
      'Finland'
    ]);
    expect(filterByName(TAGS, 'ALE', (tag) => tag)).toEqual(['Ålesund']);
    expect(filterByName(TAGS, '', (tag) => tag)).toEqual(TAGS);
  });

  it.each([
    [4, 48, NOUN, '4 of 48 tags'],
    [1, 13, { one: 'person', many: 'people' }, '1 of 13 people'],
    [0, 48, NOUN, 'No matching tags'],
    [0, 20, { one: 'folder', many: 'folders' }, 'No matching folders']
  ])('reads %i of %i as %s', (shown, total, noun, expected) => {
    expect(listFilterStatus(shown, total, noun)).toBe(expected);
  });
});
