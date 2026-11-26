import { descriptionChanges, descriptionValues } from '#lib/metadata/descriptions.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { describe, expect, it } from 'vitest';

function asset(id: string, description: string | null): AssetSummary {
  return {
    id,
    originalFileName: `${id}.jpg`,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: { description } as AssetSummary['exifInfo'],
    tags: []
  };
}

const assets = [
  asset('a', 'Fløyen'),
  asset('b', ' Fløyen '),
  asset('c', null),
  asset('d', 'Ulriken')
];

describe('descriptions', () => {
  it('counts the descriptions in a selection, most used first', () => {
    expect(descriptionValues(assets)).toEqual([
      { text: 'Fløyen', count: 2 },
      { text: '', count: 1 },
      { text: 'Ulriken', count: 1 }
    ]);
  });

  it('changes only the photos whose description differs', () => {
    expect(descriptionChanges(assets, 'Fløyen').map((change) => change.id)).toEqual(['c', 'd']);
    expect(descriptionChanges(assets, '')).toHaveLength(3);
    expect(descriptionChanges(assets, 'Ulriken')[1]).toEqual({
      id: 'b',
      from: ' Fløyen ',
      to: 'Ulriken'
    });
  });
});
