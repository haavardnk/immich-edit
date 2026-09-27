import { describe, expect, it } from 'vitest';
import { sortAssets } from './sortAssets';
import type { AssetSummary } from '$lib/types/album';

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

const takenAt = (a: AssetSummary): string | null => a.fileCreatedAt;
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
