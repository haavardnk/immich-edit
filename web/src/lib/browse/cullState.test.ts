import { session } from '#lib/stores/session.svelte.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { afterEach, describe, expect, it } from 'vitest';
import { cullState } from './cullState';

const reject: TagRef = { id: 'r', name: 'reject', value: 'immich-edit/reject' };
const red: TagRef = { id: 'l', name: 'red', value: 'immich-edit/label/red' };

describe('cullState', () => {
  afterEach(() => session.clear());

  it('reads rating, favourite, reject and label from the asset', () => {
    expect(cullState({ isFavorite: true, exifInfo: { rating: 4 }, tags: [reject, red] })).toEqual({
      rating: 4,
      isFavorite: true,
      rejected: true,
      label: 'red',
      owned: true
    });
  });

  it('defaults an unrated, untagged asset', () => {
    expect(cullState({ isFavorite: false, exifInfo: null })).toEqual({
      rating: 0,
      isFavorite: false,
      rejected: false,
      label: null,
      owned: true
    });
  });

  it('marks assets of another user as not owned', () => {
    session.set({ id: 'me', email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
    expect(cullState({ ownerId: 'other', isFavorite: false, exifInfo: null }).owned).toBe(false);
    expect(cullState({ ownerId: 'me', isFavorite: false, exifInfo: null }).owned).toBe(true);
  });
});
