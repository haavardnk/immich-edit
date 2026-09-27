import { describe, expect, it } from 'vitest';
import {
  createAssetGridLayout,
  verticalAssetIndex,
  visibleAssetRange,
  type AssetGridLayout
} from './assetGridLayout';

function asset(width: number | null, height: number | null, orientation?: string) {
  return {
    exifInfo: {
      exifImageWidth: width,
      exifImageHeight: height,
      orientation
    }
  };
}

function ratio(layout: AssetGridLayout, index: number): number {
  const box = layout.boxes[index];
  return box ? box.width / box.height : NaN;
}

describe('createAssetGridLayout', () => {
  it('preserves landscape and portrait aspect ratios in justified rows', () => {
    const layout = createAssetGridLayout(
      [asset(6000, 4000), asset(3000, 4500), asset(4000, 4000), asset(8000, 3000)],
      800,
      180,
      4
    );

    expect(layout.boxes).toHaveLength(4);
    expect(ratio(layout, 0)).toBeCloseTo(1.5);
    expect(ratio(layout, 1)).toBeCloseTo(2 / 3);
    expect(ratio(layout, 2)).toBeCloseTo(1);
    expect(ratio(layout, 3)).toBeCloseTo(8 / 3);
    expect(layout.rows.every((row) => row.endIndex > row.startIndex)).toBe(true);
  });

  it('uses a landscape fallback for missing and invalid dimensions', () => {
    const layout = createAssetGridLayout([asset(null, null), asset(0, 4000)], 800, 180, 4);

    expect(ratio(layout, 0)).toBeCloseTo(1.5);
    expect(ratio(layout, 1)).toBeCloseTo(1.5);
  });

  it('swaps dimensions for rotated EXIF orientations', () => {
    const layout = createAssetGridLayout([asset(6000, 4000, '6')], 800, 180, 4);

    expect(ratio(layout, 0)).toBeCloseTo(2 / 3);
  });

  it('returns empty geometry before the grid has a measurable width', () => {
    expect(createAssetGridLayout([asset(6000, 4000)], 0, 180, 4)).toEqual({
      boxes: [],
      rows: [],
      height: 0
    });
  });

  it('returns complete rows around the viewport', () => {
    const assets = Array.from({ length: 20 }, () => asset(6000, 4000));
    const layout = createAssetGridLayout(assets, 800, 120, 4);
    const secondRow = layout.rows[1]!;
    const range = visibleAssetRange(layout, secondRow.top, secondRow.top + secondRow.height, 0);

    expect(range).toEqual({ startIndex: secondRow.startIndex, endIndex: secondRow.endIndex });
  });

  it('moves vertically to the nearest visual column', () => {
    const layout = createAssetGridLayout(
      [asset(6000, 4000), asset(3000, 4500), asset(8000, 3000), asset(4000, 4000)],
      500,
      140,
      4
    );
    const sourceIndex = layout.rows[0]!.endIndex - 1;
    const targetIndex = verticalAssetIndex(layout, sourceIndex, 1);
    const source = layout.boxes[sourceIndex]!;
    const target = layout.boxes[targetIndex]!;
    const sourceCenter = source.left + source.width / 2;
    const targetDistance = Math.abs(target.left + target.width / 2 - sourceCenter);
    const targetRow = layout.rows[1]!;

    expect(target.row).toBe(1);
    for (let index = targetRow.startIndex; index < targetRow.endIndex; index += 1) {
      const candidate = layout.boxes[index]!;
      expect(targetDistance).toBeLessThanOrEqual(
        Math.abs(candidate.left + candidate.width / 2 - sourceCenter)
      );
    }
  });
});
