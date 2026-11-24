import { describe, expect, it } from 'vitest';
import { awayFrom } from '#lib/geotag/spots.ts';

const PUNE = { lat: 18.5204, lon: 73.8567 };

describe('awayFrom', () => {
  it('keeps photos away from the spot and drops those already there', () => {
    const located = [
      { id: 'there', lat: 18.5206, lon: 73.8569 },
      { id: 'bergen', lat: 60.3913, lon: 5.3221 }
    ];
    expect(awayFrom(['there', 'bergen', 'unplaced'], located, PUNE)).toEqual([
      'bergen',
      'unplaced'
    ]);
  });

  it('counts a photo just past 50 m as away', () => {
    expect(awayFrom(['near'], [{ id: 'near', lat: 18.521, lon: 73.8567 }], PUNE)).toEqual(['near']);
  });
});
