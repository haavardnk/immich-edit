import { describe, expect, it } from 'vitest';
import { isManagedTag, tagHasValue, toTagRef, userTags } from './managedTags';
import type { TagRef } from '#lib/types/asset.ts';

const keep: TagRef = { id: 'k', name: 'Keep', value: 'Keep' };

describe('isManagedTag', () => {
  it.each(['immich-edit', 'immich-edit/reject', 'IMMICH-EDIT/Reject', 'immich-edit/label/red'])(
    'claims %s',
    (value) => {
      expect(isManagedTag({ id: '1', name: 'x', value })).toBe(true);
    }
  );

  it('ignores unrelated tags', () => {
    expect(isManagedTag(keep)).toBe(false);
    expect(isManagedTag({ id: '1', name: 'x', value: 'immich-editor' })).toBe(false);
  });
});

describe('tagHasValue', () => {
  it('compares values case-insensitively', () => {
    expect(
      tagHasValue({ id: '1', name: 'x', value: 'IMMICH-EDIT/REJECT' }, 'immich-edit/reject')
    ).toBe(true);
    expect(tagHasValue(keep, 'immich-edit/reject')).toBe(false);
  });
});

describe('toTagRef', () => {
  it('maps summary fields', () => {
    const ref = toTagRef({
      id: 'r',
      name: 'reject',
      value: 'immich-edit/reject',
      parentId: 'p',
      color: '#fff',
      createdAt: ''
    });
    expect(ref).toEqual({
      id: 'r',
      name: 'reject',
      value: 'immich-edit/reject',
      parentId: 'p',
      color: '#fff'
    });
  });
});

describe('userTags', () => {
  it('drops the tags immich-edit manages', () => {
    const tags = [
      { id: 'a', name: 'Sky', value: 'Sky', createdAt: null },
      { id: 'b', name: 'reject', value: 'immich-edit/reject', createdAt: null }
    ];
    expect(userTags(tags).map((tag) => tag.id)).toEqual(['a']);
  });
});
