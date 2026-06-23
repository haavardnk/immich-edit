import { describe, it, expect } from 'vitest';
import { looksLikeFilename, resolveSearchMode } from './searchMode';

describe('looksLikeFilename', () => {
  it.each([
    'DSC00195',
    'IMG_1234',
    '_DSC1234',
    'P1000123',
    'file.arw',
    'photo.jpg',
    'scan.tiff',
    '12345'
  ])('treats %s as a filename', (q) => {
    expect(looksLikeFilename(q)).toBe(true);
  });

  it.each(['red car', 'sunset', 'beach at night', 'dog', '', '   '])(
    'treats %s as description',
    (q) => {
      expect(looksLikeFilename(q)).toBe(false);
    }
  );
});

describe('resolveSearchMode', () => {
  it('honors an explicit mode', () => {
    expect(resolveSearchMode('red car', 'filename')).toBe('filename');
    expect(resolveSearchMode('DSC00195', 'description')).toBe('description');
  });

  it('ignores an invalid explicit mode and auto-detects', () => {
    expect(resolveSearchMode('DSC00195', 'bogus')).toBe('filename');
    expect(resolveSearchMode('red car', null)).toBe('description');
  });
});
