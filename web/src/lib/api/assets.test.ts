import { describe, expect, it, vi } from 'vitest';
import { assetPlaceholder, assetThumbUrl, immichPreviewUrl, thumbUrl } from '#lib/api/assets.ts';
import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';

vi.mock('#lib/api/edits.ts', () => ({ listEditedAssets: vi.fn() }));

describe('Immich thumbnail URLs', () => {
  it('carry a version built from the thumbhash and the checksum', () => {
    expect(assetThumbUrl({ id: 'a', thumbhash: '1QcS+HQ/h4=', checksum: 'sha1' })).toBe(
      '/api/assets/a/thumb?size=thumbnail&c=8dfd58d57b6a2c14'
    );
    expect(immichPreviewUrl({ id: 'a', thumbhash: 'h', checksum: 's' })).toBe(
      '/api/assets/a/thumb?size=preview&c=ed0c3757f60c4582'
    );
  });

  it('change version when only the checksum changes', () => {
    const before = assetThumbUrl({ id: 'a', thumbhash: '1QcS+HQ/h4=', checksum: 'sha1' });
    const after = assetThumbUrl({ id: 'a', thumbhash: '1QcS+HQ/h4=', checksum: 'sha2' });
    expect(after).toBe('/api/assets/a/thumb?size=thumbnail&c=8dfd58d57e6a30cd');
    expect(after).not.toBe(before);
  });

  it('stay unversioned without both a thumbhash and a checksum', () => {
    expect(assetThumbUrl({ id: 'a' })).toBe('/api/assets/a/thumb?size=thumbnail');
    expect(assetThumbUrl({ id: 'a', thumbhash: 'h' })).toBe('/api/assets/a/thumb?size=thumbnail');
    expect(assetThumbUrl({ id: 'a', thumbhash: null, checksum: 's' })).toBe(
      '/api/assets/a/thumb?size=thumbnail'
    );
    expect(thumbUrl('a', 'preview', '')).toBe('/api/assets/a/thumb?size=preview');
  });

  it('share the master version between a virtual copy and its master', () => {
    const master = '0b7c3c1e-5f2a-4c8e-9d1b-2a3f4e5d6c7b';
    expect(immichPreviewUrl({ id: `${master}_1`, thumbhash: 'h', checksum: 's' })).toBe(
      `/api/assets/${master}/thumb?size=preview&c=ed0c3757f60c4582`
    );
  });
});

describe('assetPlaceholder', () => {
  const thumbhash = '1QcSHQRnh493V4dIh4eXh1h4kJUI';

  it('paints the decoded thumbhash', () => {
    expect(assetPlaceholder({ id: 'p1', thumbhash })).toMatch(
      /^url\("data:image\/png;base64,.+"\)$/
    );
  });

  it('is absent without a thumbhash', () => {
    expect(assetPlaceholder({ id: 'p2' })).toBeUndefined();
    expect(assetPlaceholder({ id: 'p2', thumbhash: null })).toBeUndefined();
  });

  it('is absent for an edited asset whose thumbnail no longer matches the hash', () => {
    editedThumbs.upsert('p3', 'edit-hash', '2026-01-01T00:00:00Z');
    expect(assetPlaceholder({ id: 'p3', thumbhash })).toBeUndefined();
    editedThumbs.remove('p3');
    expect(assetPlaceholder({ id: 'p3', thumbhash })).toBeDefined();
  });
});
