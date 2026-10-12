import { afterEach, describe, it, expect } from 'vitest';
import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';
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
  afterEach(() => {
    immichCapabilities.smartSearch = true;
    immichCapabilities.ocr = true;
  });

  it.each([
    ['red car', 'filename', 'filename'],
    ['DSC00195', 'context', 'context'],
    ['red car', 'description', 'description'],
    ['/photos/2024', 'path', 'path'],
    ['open', 'ocr', 'ocr'],
    ['DSC00195', 'bogus', 'filename'],
    ['red car', null, 'context']
  ])('resolves %s with mode %s to %s', (query, explicit, expected) => {
    expect(resolveSearchMode(query, explicit)).toBe(expected);
  });

  it('falls back from modes the Immich server lacks', () => {
    immichCapabilities.smartSearch = false;
    immichCapabilities.ocr = false;
    expect(resolveSearchMode('red car', 'context')).toBe('filename');
    expect(resolveSearchMode('red car', 'ocr')).toBe('filename');
    expect(resolveSearchMode('red car', null)).toBe('filename');
  });
});
