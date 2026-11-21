import { cssTagColor } from '#lib/browse/tagColor.ts';
import { describe, expect, it } from 'vitest';

describe('cssTagColor', () => {
  it.each([
    ['#3E8ED0', '#3E8ED0'],
    ['3e8ed0', '#3e8ed0'],
    [' #fff ', '#fff'],
    ['ffff', '#ffff'],
    ['#3e8ed0aa', '#3e8ed0aa']
  ])('accepts %s', (raw, color) => {
    expect(cssTagColor(raw)).toBe(color);
  });

  it.each([null, undefined, '', 'blue', '#12', '#12345', 'red;background:url(x)'])(
    'refuses %s',
    (raw) => {
      expect(cssTagColor(raw)).toBeNull();
    }
  );
});
