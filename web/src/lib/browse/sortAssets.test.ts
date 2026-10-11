import { describe, expect, it } from 'vitest';
import { fieldKey, sortAssets } from './sortAssets';
import type { AssetSummary } from '$lib/types/album';
import type { ExifInfo } from '$lib/types/asset';

function asset(id: string, at: string | null, copyOf?: string): AssetSummary {
  return {
    id,
    originalFileName: id,
    type: 'IMAGE',
    fileCreatedAt: at,
    updatedAt: at,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: [],
    ...(copyOf ? { copyOf } : {})
  };
}

const takenAt = fieldKey('taken');
const ids = (list: AssetSummary[]): string[] => list.map((a) => a.id);

describe('sortAssets', () => {
  const list = [
    asset('b', '2024-02-01T00:00:00Z'),
    asset('c', '2024-03-01T00:00:00Z'),
    asset('a', '2024-01-01T00:00:00Z')
  ];

  it.each([
    ['asc', ['a', 'b', 'c']],
    ['desc', ['c', 'b', 'a']]
  ] as ['asc' | 'desc', string[]][])('orders %s by timestamp', (dir, expected) => {
    expect(ids(sortAssets(list, takenAt, dir))).toEqual(expected);
  });

  it('keeps equal timestamps in their original order', () => {
    const same = [
      asset('x', '2024-01-01T00:00:00Z'),
      asset('y', '2024-01-01T00:00:00Z'),
      asset('z', '2024-01-01T00:00:00Z')
    ];
    expect(ids(sortAssets(same, takenAt, 'asc'))).toEqual(['x', 'y', 'z']);
    expect(ids(sortAssets(same, takenAt, 'desc'))).toEqual(['x', 'y', 'z']);
  });

  it.each(['asc', 'desc'] as const)('puts unusable timestamps last when %s', (dir) => {
    const mixed = [
      asset('none', null),
      asset('bad', 'not-a-date'),
      asset('b', '2024-02-01T00:00:00Z'),
      asset('a', '2024-01-01T00:00:00Z')
    ];
    expect(ids(sortAssets(mixed, takenAt, dir)).slice(2)).toEqual(['none', 'bad']);
  });

  it.each([
    ['desc', ['five', 'three', 'rejected', 'unrated']],
    ['asc', ['rejected', 'three', 'five', 'unrated']]
  ] as ['asc' | 'desc', string[]][])('orders rating %s with unrated last', (dir, expected) => {
    const rated = (id: string, rating: number | null): AssetSummary => ({
      ...asset(id, null),
      exifInfo: { rating } as ExifInfo
    });
    const list = [
      rated('three', 3),
      rated('unrated', null),
      rated('five', 5),
      rated('rejected', -1)
    ];
    expect(ids(sortAssets(list, fieldKey('rating'), dir))).toEqual(expected);
  });

  it('keeps virtual copies attached to their master', () => {
    const withCopies = [
      asset('b', '2024-02-01T00:00:00Z'),
      asset('b~1', '2024-02-01T00:00:00Z', 'b'),
      asset('b~2', '2024-02-01T00:00:00Z', 'b'),
      asset('a', '2024-01-01T00:00:00Z'),
      asset('a~1', '2024-01-01T00:00:00Z', 'a')
    ];
    expect(ids(sortAssets(withCopies, takenAt, 'asc'))).toEqual(['a', 'a~1', 'b', 'b~1', 'b~2']);
    expect(ids(sortAssets(withCopies, takenAt, 'desc'))).toEqual(['b', 'b~1', 'b~2', 'a', 'a~1']);
  });
});
