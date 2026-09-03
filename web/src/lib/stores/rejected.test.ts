import { describe, it, expect, beforeEach } from 'vitest';
import { rejected } from './rejected.svelte';
import type { AssetSummary } from '$lib/types/album';
import type { TagRef } from '$lib/types/asset';

const rejectTag: TagRef = { id: 'r', name: 'reject', value: 'immich-edit/reject' };

function asset(id: string, tags: TagRef[] = []): AssetSummary {
  return {
    id,
    originalFileName: id,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags
  };
}

describe('rejected store', () => {
  beforeEach(() => rejected.reset());

  it('stamps the reject tag onto matching assets', () => {
    rejected.add('a', rejectTag);
    const out = rejected.stamp([asset('a'), asset('b')]);
    expect(out[0]?.tags).toContainEqual(rejectTag);
    expect(out[1]?.tags).toHaveLength(0);
  });

  it('does not duplicate an existing reject tag', () => {
    rejected.add('a', rejectTag);
    const out = rejected.stamp([asset('a', [rejectTag])]);
    expect(out[0]?.tags).toHaveLength(1);
  });

  it('returns input unchanged when empty', () => {
    const input = [asset('a')];
    expect(rejected.stamp(input)).toBe(input);
  });

  it('remove drops the id from stamping', () => {
    rejected.add('a', rejectTag);
    rejected.remove('a');
    const out = rejected.stamp([asset('a')]);
    expect(out[0]?.tags).toHaveLength(0);
  });
});
