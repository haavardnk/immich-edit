import { describe, expect, it } from 'vitest';
import { TILE_INFO_OPTIONS, tileInfoLabel, viewSummary, type ViewState } from './viewOptions';

describe('thumbnail info options', () => {
  it('lists every mode in cycle order with a capitalised label', () => {
    expect(TILE_INFO_OPTIONS).toEqual([
      { value: 'none', label: 'Hover only' },
      { value: 'badges', label: 'Badges' },
      { value: 'full', label: 'Name and date' }
    ]);
  });

  it('names the current mode for the toggle', () => {
    expect(tileInfoLabel('full')).toBe('Thumbnail info: name and date');
  });
});

describe('viewSummary', () => {
  const view: ViewState = {
    basis: 'capture',
    field: 'taken',
    dir: 'desc',
    size: 'lg',
    info: 'badges'
  };

  it('reads sort, order, size and thumbnail info on one line', () => {
    expect(viewSummary(view)).toBe('Date taken, newest · L · Badges');
  });

  it('names the order of each sort field', () => {
    expect(viewSummary({ ...view, field: 'rating', dir: 'asc', size: 'xl', info: 'full' })).toBe(
      'Rating, lowest · XL · Name and date'
    );
    expect(viewSummary({ ...view, field: 'size', size: 'sm', info: 'none' })).toBe(
      'Size, largest · S · Hover only'
    );
  });

  it('uses the edit order where photos sort by their last edit', () => {
    expect(viewSummary({ ...view, basis: 'edit', dir: 'asc', size: 'md' })).toBe(
      'Oldest edit · M · Badges'
    );
  });

  it('leaves the sort out where the view has no sort', () => {
    expect(viewSummary({ ...view, basis: null })).toBe('L · Badges');
  });
});
