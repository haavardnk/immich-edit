import { TAG_SWATCHES, cssTagColor, sameTagColor, tagSwatches } from '#lib/browse/tagColor.ts';
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

describe('tag swatches', () => {
  it('compares colors by value, ignoring case and the hash', () => {
    expect(sameTagColor('#3E8ED0', '3e8ed0')).toBe(true);
    expect(sameTagColor(null, undefined)).toBe(true);
    expect(sameTagColor('#3e8ed0', null)).toBe(false);
  });

  it('adds a color set in Immich as a ninth swatch so a save keeps it', () => {
    expect(tagSwatches(null)).toEqual(TAG_SWATCHES);
    expect(tagSwatches('#3E8ED0')).toEqual(TAG_SWATCHES);
    const custom = tagSwatches('#123456');
    expect(custom).toHaveLength(TAG_SWATCHES.length + 1);
    expect(custom.at(-1)).toEqual({ value: '#123456', label: 'Color from Immich' });
  });
});
