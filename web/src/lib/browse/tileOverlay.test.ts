import { describe, expect, it } from 'vitest';
import {
  DEFAULT_TILE_OVERLAY,
  MAX_DETAILS,
  OVERLAY_DETAILS,
  OVERLAY_ICONS,
  detailFit,
  isDefaultOverlay,
  membershipNeed,
  nextTileInfo,
  tileInfoFrom,
  tileOverlayFrom,
  withDetail,
  withIcon,
  type TileOverlay
} from './tileOverlay';

describe('tile info modes', () => {
  it('reads the old badges mode as icons and rejects unknown modes', () => {
    expect(tileInfoFrom('badges')).toBe('icons');
    expect(tileInfoFrom('full')).toBe('full');
    expect(tileInfoFrom('all')).toBeNull();
    expect(tileInfoFrom(undefined)).toBeNull();
  });

  it('cycles hover only, icons and full', () => {
    expect(nextTileInfo('none')).toBe('icons');
    expect(nextTileInfo('icons')).toBe('full');
    expect(nextTileInfo('full')).toBe('none');
  });
});

describe('tile overlay choice', () => {
  it('defaults to the culling marks with name and date', () => {
    expect(DEFAULT_TILE_OVERLAY).toEqual({
      icons: ['edited', 'favorite', 'rejected', 'label', 'rating', 'copy'],
      details: ['filename', 'taken']
    });
    expect(isDefaultOverlay(DEFAULT_TILE_OVERLAY)).toBe(true);
  });

  it('parses stored keys into catalog order, dropping unknown keys and extra details', () => {
    expect(
      tileOverlayFrom({
        icons: ['format', 'people', 'favorite', 'favorite'],
        details: ['size', 'lens', 'camera', 'exposure', 'filename', 'nope']
      })
    ).toEqual({
      icons: ['favorite', 'format'],
      details: ['filename', 'exposure', 'camera', 'lens']
    });
    expect(tileOverlayFrom({})).toEqual({ icons: [], details: [] });
    expect(tileOverlayFrom(null)).toBeNull();
    expect(tileOverlayFrom([])).toBeNull();
  });

  it('toggles an icon in catalog order without a cap', () => {
    const none: TileOverlay = { icons: [], details: [] };
    const all = [...OVERLAY_ICONS]
      .reverse()
      .reduce((next, icon) => withIcon(next, icon, true), none);
    expect(all.icons).toEqual(OVERLAY_ICONS);
    expect(withIcon(all, 'rating', false).icons).not.toContain('rating');
    expect(withIcon(none, 'tags', false).icons).toEqual([]);
    expect(none.icons).toEqual([]);
  });

  it('caps details and keeps catalog order', () => {
    let overlay = withDetail(DEFAULT_TILE_OVERLAY, 'lens', true);
    overlay = withDetail(overlay, 'place', true);
    expect(overlay.details).toEqual(['filename', 'taken', 'place', 'lens']);
    expect(overlay.details).toHaveLength(MAX_DETAILS);
    expect(withDetail(overlay, 'size', true)).toBe(overlay);
    expect(withDetail(overlay, 'taken', false).details).toEqual(['filename', 'place', 'lens']);
    expect(OVERLAY_DETAILS).toHaveLength(10);
  });
});

describe('detail fit', () => {
  it.each([
    [72, 0],
    [89, 0],
    [90, 1],
    [139, 1],
    [140, 2],
    [199, 2],
    [200, 3],
    [279, 3],
    [280, 4],
    [600, 4]
  ])('fits %i px to %i lines', (height, lines) => {
    expect(detailFit(height)).toBe(lines);
  });
});

describe('membership need', () => {
  it('asks for nothing in hover only mode', () => {
    const overlay = { icons: ['albums', 'tags'], details: ['albums'] } as const;
    expect(membershipNeed('none', overlay)).toEqual({ albums: false, tags: false });
  });

  it('asks for a kind when either list shows it', () => {
    expect(membershipNeed('icons', { icons: ['albums'], details: ['tags'] })).toEqual({
      albums: true,
      tags: true
    });
    expect(membershipNeed('full', DEFAULT_TILE_OVERLAY)).toEqual({ albums: false, tags: false });
  });
});
