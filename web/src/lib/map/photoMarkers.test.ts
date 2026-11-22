import { markerIdentity, pickOrder, shifted } from '#lib/map/photoMarkers.ts';
import { describe, expect, it } from 'vitest';

describe('photo markers', () => {
  it('names a marker by what it shows, not where a tile put it', () => {
    expect(markerIdentity(null, 3, 1, true)).toBe(markerIdentity(null, 3, 1, true));
    expect(markerIdentity(7, 0, 2, false)).not.toBe(markerIdentity(7, 0, 3, false));
    expect(markerIdentity(7, 0, 2, false)).not.toBe(markerIdentity(7, 0, 2, true));
  });

  it('moves a drawn marker only when its point changed', () => {
    expect(shifted([10.7522, 59.9139], [10.7522, 59.9139])).toBe(false);
    expect(shifted([10.7522, 59.9139], [5.324, 60.3975])).toBe(true);
  });

  it('orders cluster leaves by their marker order', () => {
    expect(
      pickOrder([
        {
          type: 'Feature',
          geometry: { type: 'Point', coordinates: [0, 0] },
          properties: { order: 2 }
        },
        {
          type: 'Feature',
          geometry: { type: 'Point', coordinates: [0, 0] },
          properties: { order: 0 }
        }
      ])
    ).toEqual([0, 2]);
  });
});
