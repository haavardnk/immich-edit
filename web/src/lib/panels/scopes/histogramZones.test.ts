import { describe, expect, it } from 'vitest';
import { HISTOGRAM_ZONES, zoneAt, zoneValue } from './histogramZones';

describe('histogram zones', () => {
  it('maps the five tone zones left to right', () => {
    expect(HISTOGRAM_ZONES.map((zone) => zone.param.id)).toEqual([
      'blacks',
      'shadows',
      'exposure',
      'highlights',
      'whites'
    ]);
    expect([0, 0.2, 0.5, 0.8, 0.95, 1].map((x) => zoneAt(x).param.id)).toEqual([
      'blacks',
      'shadows',
      'exposure',
      'highlights',
      'whites',
      'whites'
    ]);
  });

  it('moves half the range across the full width and clamps', () => {
    const shadows = zoneAt(0.2).param;
    expect(zoneValue(shadows, 0, 50, 200)).toBe(25);
    expect(zoneValue(shadows, 90, 200, 200)).toBe(100);
    const exposure = zoneAt(0.5).param;
    expect(zoneValue(exposure, 0, -40, 200)).toBe(-1);
  });
});
