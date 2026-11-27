import type { PhotoMatch } from '#lib/geotag/match.ts';
import { distanceText } from '#lib/geotag/spots.ts';
import { matchStatus, matchTimes } from '#lib/geotag/trackRows.ts';
import { describe, expect, it } from 'vitest';

const MIN = 60_000;
const T0 = Date.parse('2026-10-31T08:15:04Z');
const TRACK = { kind: 'placed', lat: 60, lon: 6, gapMs: 0 } as const;

function match(outcome: PhotoMatch['outcome'], time = T0): PhotoMatch {
  return { id: 'a', time, outcome };
}

describe('track rows', () => {
  it('says how far a located photo is from the track', () => {
    const kept = match({ kind: 'kept', at: { lat: 60.00036, lon: 6 }, track: TRACK });
    expect(matchStatus(kept)).toEqual({
      text: '40 m from track',
      tone: 'kept'
    });
    const late = match({
      kind: 'kept',
      at: { lat: 60, lon: 6 },
      track: { kind: 'outside', reason: 'after', gapMs: 40 * MIN }
    });
    expect(matchStatus(late).text).toBe('40 min after end');
  });

  it('says how far a moved photo travels', () => {
    const moved = match({ ...TRACK, from: { lat: 60.00036, lon: 6 } });
    expect(matchStatus(moved)).toEqual({ text: 'Moves 40 m', tone: 'ok' });
    expect(matchStatus(match({ ...TRACK, from: null })).text).toBe('');
  });

  it('shows one time unless the shift changes it', () => {
    const base = { instant: T0, recorded: true };
    expect(matchTimes(base, match({ ...TRACK, from: null }), 'UTC')).toBe('08:15:04');
    const shifted = match({ ...TRACK, from: null }, T0 + 60 * MIN);
    expect(matchTimes(base, shifted, 'UTC')).toBe('08:15:04 → 09:15:04');
  });

  it('rounds distances to what a map can tell apart', () => {
    expect(distanceText(3)).toBe('10 m');
    expect(distanceText(44)).toBe('40 m');
    expect(distanceText(996)).toBe('1.0 km');
    expect(distanceText(18_420)).toBe('18.4 km');
  });
});
