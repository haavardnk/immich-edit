import { describe, it, expect } from 'vitest';
import {
  isRejectTag,
  isManagedTag,
  isRejected,
  toTagRef,
  addRejectTag,
  removeRejectTag,
  setRejectedTags,
  REJECT_TAG_VALUE
} from './reject';
import type { TagRef } from './types/asset';

const reject: TagRef = { id: 'r', name: 'reject', value: 'immich-edit/reject' };
const keep: TagRef = { id: 'k', name: 'Keep', value: 'Keep' };

describe('isRejectTag', () => {
  it('matches by value case-insensitively', () => {
    expect(isRejectTag({ id: '1', name: 'x', value: 'immich-edit/reject' })).toBe(true);
    expect(isRejectTag({ id: '1', name: 'x', value: 'IMMICH-EDIT/REJECT' })).toBe(true);
  });

  it('does not match a bare reject name', () => {
    expect(isRejectTag({ id: '1', name: 'reject', value: 'reject' })).toBe(false);
  });

  it('rejects non-matching tags', () => {
    expect(isRejectTag(keep)).toBe(false);
  });
});

describe('isManagedTag', () => {
  it('matches the namespace root and children', () => {
    expect(isManagedTag({ id: '1', name: 'x', value: 'immich-edit' })).toBe(true);
    expect(isManagedTag({ id: '1', name: 'x', value: 'immich-edit/reject' })).toBe(true);
    expect(isManagedTag({ id: '1', name: 'x', value: 'IMMICH-EDIT/Reject' })).toBe(true);
  });

  it('ignores unrelated tags', () => {
    expect(isManagedTag(keep)).toBe(false);
    expect(isManagedTag({ id: '1', name: 'x', value: 'immich-editor' })).toBe(false);
  });
});

describe('isRejected', () => {
  it('detects a reject tag among others', () => {
    expect(isRejected({ tags: [keep, reject] })).toBe(true);
  });

  it('handles missing tags', () => {
    expect(isRejected({ tags: null })).toBe(false);
    expect(isRejected({})).toBe(false);
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

describe('add/remove/set reject tag', () => {
  it('adds once', () => {
    const out = addRejectTag([keep], reject);
    expect(out).toHaveLength(2);
    expect(addRejectTag(out, reject)).toHaveLength(2);
  });

  it('removes reject tags', () => {
    expect(removeRejectTag([keep, reject])).toEqual([keep]);
  });

  it('sets rejected state', () => {
    expect(setRejectedTags([keep], reject, true)).toContainEqual(reject);
    expect(setRejectedTags([keep, reject], reject, false)).toEqual([keep]);
  });
});

describe('REJECT_TAG_VALUE', () => {
  it('is immich-edit/reject', () => {
    expect(REJECT_TAG_VALUE).toBe('immich-edit/reject');
  });
});
