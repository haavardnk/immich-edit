import { forgetMap, mapMemory, mapStart, type MapMemory } from '#lib/map/mapMemory.ts';
import { describe, expect, it } from 'vitest';

const MEMORY: MapMemory = {
  view: { lat: 60, lon: 5, zoom: 9 },
  filter: { favorite: true },
  tab: 'unplaced',
  panel: false
};

describe('map memory', () => {
  it('restores the last view and filters on a bare /map', () => {
    expect(mapStart(new URLSearchParams(''), MEMORY)).toEqual({
      view: MEMORY.view,
      filter: { favorite: true }
    });
  });

  it('lets a link with a view or filters win', () => {
    const linked = mapStart(new URLSearchParams('at=59.9,10.7,12'), MEMORY);
    expect(linked.view).toEqual({ lat: 59.9, lon: 10.7, zoom: 12 });
    expect(linked.filter.favorite).toBe(false);
    expect(mapStart(new URLSearchParams('after=2024-01-01'), MEMORY).filter.after).toBe(
      '2024-01-01'
    );
  });

  it('forgets everything when the map is left for its parent', () => {
    Object.assign(mapMemory, MEMORY);
    forgetMap();
    expect(mapMemory).toEqual({ view: null, filter: {}, tab: 'view', panel: null });
  });
});
