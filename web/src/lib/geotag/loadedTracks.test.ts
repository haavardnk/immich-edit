import { describe, expect, it } from 'vitest';
import { loadedTrack, trackDetail, trackLogOf, trackName, trackShiftText } from './loadedTracks';
import type { TrackPoint } from './trackFile';

const HOUR = 3_600_000;
const START = Date.UTC(2026, 9, 31, 9, 2);

function line(start: number, hours: number): TrackPoint[] {
  return [
    { lat: 60, lon: 7, time: start },
    { lat: 60.1, lon: 7, time: start + hours * HOUR }
  ];
}

describe('loaded tracks', () => {
  it('names a track by its name or its file', () => {
    expect(trackName(loadedTrack('hike.gpx', 'Hardangervidda', [line(START, 1)], 1))).toBe(
      'Hardangervidda'
    );
    expect(trackName(loadedTrack('hike.gpx', null, [line(START, 1)], 2))).toBe('hike.gpx');
  });

  it('moves only the shifted track in the combined log', () => {
    const first = loadedTrack('a.gpx', null, [line(START, 1)], 1);
    const second = {
      ...loadedTrack('b.gpx', null, [line(START + 4 * HOUR, 1)], 2),
      offsetMs: -HOUR
    };
    const log = trackLogOf([first, second]);
    expect(log.start).toBe(START);
    expect(log.end).toBe(START + 4 * HOUR);
  });

  it('describes the day, hours, distance and shift', () => {
    const track = { ...loadedTrack('a.gpx', null, [line(START, 6.5)], 1), offsetMs: -HOUR };
    expect(trackDetail([track], 'Europe/Oslo')).toBe(
      `31 Oct · 10:02–16:32 · ${(track.metres / 1000).toFixed(1)} km · Shift −1:00:00`
    );
  });

  it('spans days when the tracks do', () => {
    const first = loadedTrack('a.gpx', null, [line(START, 1)], 1);
    const second = loadedTrack('b.gpx', null, [line(START + 24 * HOUR, 1)], 2);
    expect(trackDetail([first, second], 'UTC')).toMatch(/^31 Oct – 1 Nov · /);
    expect(trackDetail([], 'UTC')).toBe('');
  });

  it('counts tracks with different shifts', () => {
    const first = { ...loadedTrack('a.gpx', null, [line(START, 1)], 1), offsetMs: HOUR };
    const second = { ...loadedTrack('b.gpx', null, [line(START, 1)], 2), offsetMs: -HOUR };
    expect(trackShiftText([first, second])).toBe('2 tracks shifted');
    expect(trackShiftText([{ ...first, offsetMs: 0 }])).toBe('');
  });
});
