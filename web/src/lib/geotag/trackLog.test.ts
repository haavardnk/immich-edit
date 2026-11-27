import type { TrackPoint } from '#lib/geotag/trackFile.ts';
import { countMatches, matchPhotos } from '#lib/geotag/match.ts';
import { TrackLog } from '#lib/geotag/trackLog.ts';
import { describe, expect, it } from 'vitest';

const T0 = Date.parse('2026-10-31T08:00:00Z');
const MIN = 60_000;

function at(minute: number, lat: number, lon: number): TrackPoint {
  return { lat, lon, time: T0 + minute * MIN };
}

const WALK = new TrackLog([
  [at(0, 60, 6), at(10, 60.01, 6), at(20, 60.02, 6)],
  [at(60, 60.1, 6.1), at(100, 60.1, 6.1)]
]);
const OPTIONS = { toleranceMs: 5 * MIN, interpolate: true };
const HOME = { lat: 59.9, lon: 10.7 };

describe('track lookup', () => {
  it('interpolates between points in one segment', () => {
    const found = WALK.locate(T0 + 5 * MIN, OPTIONS);
    expect(found).toMatchObject({ kind: 'placed', gapMs: 5 * MIN });
    if (found.kind === 'placed') expect(found.lat).toBeCloseTo(60.005, 6);
  });

  it('takes the nearest point beyond the ends within the tolerance', () => {
    expect(WALK.locate(T0 - 3 * MIN, OPTIONS)).toMatchObject({ kind: 'placed', lat: 60 });
    expect(WALK.locate(T0 - 6 * MIN, OPTIONS)).toEqual({
      kind: 'outside',
      reason: 'before',
      gapMs: 6 * MIN
    });
    expect(WALK.locate(T0 + 107 * MIN, OPTIONS)).toMatchObject({
      kind: 'outside',
      reason: 'after'
    });
  });

  it('does not bridge segments, but bridges a stationary pause', () => {
    expect(WALK.locate(T0 + 40 * MIN, OPTIONS)).toEqual({
      kind: 'outside',
      reason: 'gap',
      gapMs: 20 * MIN
    });
    expect(WALK.locate(T0 + 23 * MIN, OPTIONS)).toMatchObject({ kind: 'placed', lat: 60.02 });
    expect(WALK.locate(T0 + 80 * MIN, OPTIONS)).toMatchObject({ kind: 'placed', lat: 60.1 });
  });

  it('bridges a long gap of a drawn track whose pace stays steady', () => {
    const drawn = new TrackLog([
      [at(0, 60, 6), at(15, 60.03, 6), at(30, 60.06, 6), at(45, 60.09, 6)]
    ]);
    const placed = drawn.locate(T0 + 22 * MIN + 30_000, OPTIONS);
    expect(placed.kind).toBe('placed');
    if (placed.kind === 'placed') expect(placed.lat).toBeCloseTo(60.045, 4);
  });

  it('does not bridge a long gap where the logger stood still or jumped', () => {
    const lunch = new TrackLog([
      [at(0, 60, 6), at(5, 60.01, 6), at(65, 60.012, 6), at(70, 60.022, 6)]
    ]);
    expect(lunch.locate(T0 + 35 * MIN, OPTIONS)).toMatchObject({ kind: 'outside', reason: 'gap' });
    const jump = new TrackLog([[at(0, 60, 6), at(5, 60.01, 6), at(25, 61, 6), at(30, 61.01, 6)]]);
    expect(jump.locate(T0 + 15 * MIN, OPTIONS)).toMatchObject({ kind: 'outside', reason: 'gap' });
  });

  it('only takes the nearest point when interpolation is off', () => {
    const found = WALK.locate(T0 + 4 * MIN, { ...OPTIONS, interpolate: false });
    expect(found).toMatchObject({ kind: 'placed', lat: 60, gapMs: 4 * MIN });
  });

  it('draws one line per segment and measures the walk', () => {
    expect(WALK.path().map((line) => line.length)).toEqual([3, 2]);
    expect(WALK.lengthMetres()).toBeGreaterThan(2_200);
    expect(WALK.lengthMetres()).toBeLessThan(2_300);
  });
});

describe('matching photos', () => {
  const photos = [
    { id: 'a', base: T0 + 65 * MIN, shiftMs: -60 * MIN, at: null },
    { id: 'b', base: T0 + 75 * MIN, shiftMs: -60 * MIN, at: HOME },
    { id: 'c', base: null, shiftMs: -60 * MIN, at: null },
    { id: 'd', base: T0 + 300 * MIN, shiftMs: -60 * MIN, at: null }
  ];

  it('places photos, keeps located ones and explains the rest', () => {
    const matches = matchPhotos(photos, WALK, { ...OPTIONS, replace: false });
    expect(matches.map((match) => match.outcome.kind)).toEqual([
      'placed',
      'kept',
      'untimed',
      'outside'
    ]);
    expect(matches[0]?.time).toBe(T0 + 5 * MIN);
    expect(countMatches(matches)).toEqual({ placed: 1, outside: 1, kept: 1, untimed: 1 });
    expect(matches[1]?.outcome).toMatchObject({
      kind: 'kept',
      at: HOME,
      track: { kind: 'placed' }
    });
    const unshifted = photos.map((photo) => ({ ...photo, shiftMs: 0 }));
    const replaced = matchPhotos(unshifted, WALK, { ...OPTIONS, replace: true });
    expect(replaced[1]?.outcome).toMatchObject({ kind: 'placed', from: HOME });
  });
});
