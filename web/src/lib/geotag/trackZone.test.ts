import { describe, expect, it } from 'vitest';
import { trackZone, zoneAt } from '#lib/geotag/trackZone.ts';

const point = (lat: number, lon: number) => ({ lat, lon, time: 0 });

describe('trackZone', () => {
  it('names the zone where the track was recorded', () => {
    expect(zoneAt(-2.0, 35.17)).toBe('Africa/Dar_es_Salaam');
    expect(trackZone([[point(60.1, 6.7), point(60.12, 6.72), point(60.13, 6.73)]])).toBe(
      'Europe/Oslo'
    );
  });

  it('gives nothing for an empty track or bad coordinates', () => {
    expect(trackZone([])).toBeNull();
    expect(zoneAt(200, 0)).toBeNull();
  });
});
