import { clockDrift, clocksText, correctionText, parseCorrection } from '#lib/geotag/clockText.ts';
import { describe, expect, it } from 'vitest';

const H = 3_600_000;
const M = 60_000;

describe('clock correction text', () => {
  it('writes corrections as signed h:mm:ss', () => {
    expect(correctionText(-(H + 2 * M + 14_000))).toBe('−1:02:14');
    expect(correctionText(30_000)).toBe('+0:00:30');
    expect(correctionText(0)).toBe('+0:00:00');
  });

  it('reads typed corrections', () => {
    expect(parseCorrection('-1:02:14')).toBe(-(H + 2 * M + 14_000));
    expect(parseCorrection('−1:02:14')).toBe(-(H + 2 * M + 14_000));
    expect(parseCorrection('+2')).toBe(2 * H);
    expect(parseCorrection('0:45')).toBe(45 * M);
    expect(parseCorrection('1:60')).toBeNull();
    expect(parseCorrection('25')).toBeNull();
    expect(parseCorrection('soon')).toBeNull();
  });

  it('says which way the camera clock is off', () => {
    expect(clockDrift(-(H + 2 * M + 14_000))).toBe('Camera 1 h 02 min 14 s fast');
    expect(clockDrift(3 * M)).toBe('Camera 03 min slow');
    expect(clockDrift(0)).toBe('No correction');
  });

  it('names each corrected camera and counts the rest', () => {
    expect(clocksText([{ label: 'ILCE-7M3', ms: 0 }])).toBe('No correction');
    expect(clocksText([{ label: 'ILCE-7M3', ms: -3_600_000 }])).toBe('Camera 1 h fast');
    expect(
      clocksText([
        { label: 'ILCE-7M3', ms: -(4 * 3600 + 59 * 60 + 38) * 1000 },
        { label: 'ILCE-7M4', ms: 0 },
        { label: 'iPhone 15 Pro', ms: 0 }
      ])
    ).toBe('ILCE-7M3 4 h 59 min 38 s fast · 2 cameras unchanged');
    expect(
      clocksText([
        { label: 'A', ms: 0 },
        { label: 'B', ms: 0 }
      ])
    ).toBe('No correction');
  });
});
