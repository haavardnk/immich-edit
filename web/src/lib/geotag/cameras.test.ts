import type { AssetSummary } from '#lib/types/album.ts';
import { describe, expect, it } from 'vitest';
import { cameraKey, cameraLabel, camerasOf } from './cameras';

function shot(make: string | null, model: string | null): AssetSummary {
  return { exifInfo: { make, model } } as AssetSummary;
}

describe('cameras', () => {
  it('groups photos by make and model, most photos first', () => {
    const cameras = camerasOf([
      shot('SONY', 'ILCE-7M3'),
      shot('Apple', 'iPhone 15 Pro'),
      shot('SONY', 'ILCE-7M3'),
      shot(null, null)
    ]);
    expect(cameras.map(({ label, count }) => [label, count])).toEqual([
      ['ILCE-7M3', 2],
      ['iPhone 15 Pro', 1],
      ['Unknown camera', 1]
    ]);
  });

  it('keeps models of different makes apart and names a camera without a model by its make', () => {
    expect(cameraKey(shot('A', 'X'))).not.toBe(cameraKey(shot('B', 'X')));
    expect(cameraLabel(cameraKey(shot('Fujifilm', null)))).toBe('Fujifilm');
  });
});
