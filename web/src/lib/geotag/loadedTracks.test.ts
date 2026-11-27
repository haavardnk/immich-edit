import { describe, expect, it } from 'vitest';
import { loadedTrack, trackDetail, trackLogOf, trackName, trackShiftText } from './loadedTracks';
import type { TrackPoint } from './trackFile';

function loaded(fileName: string, name: string | null, segments: TrackPoint[][], id: number) {
  return loadedTrack(fileName, { name, segments, utc: true }, id);
}

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
    expect(trackName(loaded('hike.gpx', 'Hardangervidda', [line(START, 1)], 1))).toBe(
      'Hardangervidda'
    );
    expect(trackName(loaded('hike.gpx', null, [line(START, 1)], 2))).toBe('hike.gpx');
  });

  it('moves only the shifted track in the combined log', () => {
    const first = loaded('a.gpx', null, [line(START, 1)], 1);
    const second = {
      ...loaded('b.gpx', null, [line(START + 4 * HOUR, 1)], 2),
      offsetMs: -HOUR
    };
    const log = trackLogOf([first, second]);
    expect(log.start).toBe(START);
    expect(log.end).toBe(START + 4 * HOUR);
  });

  it('describes the day, hours, distance and shift', () => {
    const track = { ...loaded('a.gpx', null, [line(START, 6.5)], 1), offsetMs: -HOUR };
    expect(trackDetail([track], 'Europe/Oslo')).toBe(
      `31 Oct · 09:02–15:32 · ${(track.metres / 1000).toFixed(1)} km · −1:00:00`
    );
  });

  it('spans days when the tracks do', () => {
    const first = loaded('a.gpx', null, [line(START, 1)], 1);
    const second = loaded('b.gpx', null, [line(START + 24 * HOUR, 1)], 2);
    expect(trackDetail([first, second], 'UTC')).toMatch(/^31 Oct – 1 Nov · /);
    expect(trackDetail([], 'UTC')).toBe('');
  });

  it('counts tracks with different changes', () => {
    const first = { ...loaded('a.gpx', null, [line(START, 1)], 1), offsetMs: HOUR };
    const second = { ...loaded('b.gpx', null, [line(START, 1)], 2), offsetMs: -HOUR };
    expect(trackShiftText([first, second])).toBe('2 tracks changed');
    expect(trackShiftText([{ ...first, offsetMs: 0 }])).toBe('');
  });

  it('reads a track logged in local time in its time zone', () => {
    const track = { ...loaded('a.gpx', null, [line(START, 1)], 1), zone: 'Africa/Dar_es_Salaam' };
    const log = trackLogOf([track]);
    expect(log.start).toBe(START - 3 * HOUR);
    expect(trackShiftText([{ ...track, offsetMs: 120_000 }])).toBe('Dar es Salaam · +0:02:00');
    expect(trackDetail([track], 'Africa/Dar_es_Salaam')).toMatch(/^31 Oct · 09:02–10:02 · /);
  });
});
