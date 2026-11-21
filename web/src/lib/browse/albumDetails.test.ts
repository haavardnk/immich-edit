import { album } from '#lib/testing/albums.ts';
import { describe, expect, it } from 'vitest';
import {
  ALBUM_DESCRIPTION_MAX,
  ALBUM_NAME_MAX,
  albumDetailsPatch,
  charCount,
  validAlbumDetails
} from './albumDetails';

describe('validAlbumDetails', () => {
  it('needs a name within the caps', () => {
    expect(validAlbumDetails('Norway', '')).toBe(true);
    expect(validAlbumDetails('   ', '')).toBe(false);
    expect(validAlbumDetails('n'.repeat(ALBUM_NAME_MAX), '')).toBe(true);
    expect(validAlbumDetails('n'.repeat(ALBUM_NAME_MAX + 1), '')).toBe(false);
    expect(validAlbumDetails('Norway', 'd'.repeat(ALBUM_DESCRIPTION_MAX + 1))).toBe(false);
  });

  it('counts characters, not UTF-16 units', () => {
    expect(charCount('🏔️')).toBe(2);
    expect(validAlbumDetails('Norway', '🏔'.repeat(ALBUM_DESCRIPTION_MAX))).toBe(true);
  });
});

describe('albumDetailsPatch', () => {
  const norway = album('a', { albumName: 'Norway', description: 'West coast' });

  it('sends only what changed, trimmed', () => {
    expect(albumDetailsPatch(norway, ' Norway ', 'West coast ')).toBeNull();
    expect(albumDetailsPatch(norway, 'Bergen', 'West coast')).toEqual({ albumName: 'Bergen' });
    expect(albumDetailsPatch(norway, 'Norway', ' ')).toEqual({ description: '' });
  });
});
