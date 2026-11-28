import { describe, expect, it } from 'vitest';
import { tileLayout } from './tileLayout';
import { DEFAULT_TILE_OVERLAY, withIcon } from './tileOverlay';

describe('tileLayout', () => {
  it('fits detail lines to the tile height', () => {
    expect(tileLayout('full', DEFAULT_TILE_OVERLAY, { width: 200, height: 72 }).lines).toBe(0);
    expect(tileLayout('full', DEFAULT_TILE_OVERLAY, { width: 200, height: 150 }).lines).toBe(2);
    expect(tileLayout('full', DEFAULT_TILE_OVERLAY, { width: 200, height: 320 }).lines).toBe(4);
  });

  it('pins the stars only in full mode on tall tiles that show ratings', () => {
    const tall = { width: 200, height: 140 };
    expect(tileLayout('full', DEFAULT_TILE_OVERLAY, tall).pinnedStars).toBe(true);
    expect(tileLayout('icons', DEFAULT_TILE_OVERLAY, tall).pinnedStars).toBe(false);
    expect(tileLayout('full', DEFAULT_TILE_OVERLAY, { width: 200, height: 139 }).pinnedStars).toBe(
      false
    );
    expect(
      tileLayout('full', withIcon(DEFAULT_TILE_OVERLAY, 'rating', false), tall).pinnedStars
    ).toBe(false);
  });

  it('drops counts, stacks the last row and tightens the band on small tiles', () => {
    expect(tileLayout('icons', DEFAULT_TILE_OVERLAY, { width: 200, height: 150 })).toMatchObject({
      counts: true,
      stacked: false,
      low: false
    });
    expect(tileLayout('icons', DEFAULT_TILE_OVERLAY, { width: 89, height: 72 })).toMatchObject({
      counts: false,
      stacked: true,
      low: true
    });
    expect(tileLayout('icons', DEFAULT_TILE_OVERLAY, { width: 119, height: 79 }).counts).toBe(
      false
    );
  });
});
