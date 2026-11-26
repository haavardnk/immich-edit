import {
  cameraShiftText,
  clocksText,
  correctionText,
  parseCorrection
} from '#lib/metadata/clockText.ts';
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
    expect(parseCorrection('')).toBeNull();
  });

  it('writes and reads days when the limit allows them', () => {
    const year = 365 * 24 * H;
    expect(correctionText(3 * 24 * H + 4 * H)).toBe('+3 d 4:00:00');
    expect(parseCorrection('+3 d 4:00:00', year)).toBe(3 * 24 * H + 4 * H);
    expect(parseCorrection('-10 d', year)).toBe(-10 * 24 * H);
    expect(parseCorrection('3 d 24:00', year)).toBeNull();
    expect(parseCorrection('3 d')).toBeNull();
    expect(parseCorrection('400 d', year)).toBeNull();
  });

  it('writes a camera shift as the stepper does, or unchanged', () => {
    expect(cameraShiftText(-(H + 2 * M + 14_000))).toBe('−1:02:14');
    expect(cameraShiftText(2 * 24 * H + H)).toBe('+2 d 1:00:00');
    expect(cameraShiftText(0)).toBe('unchanged');
    expect(cameraShiftText(400)).toBe('unchanged');
  });

  it('names each shifted camera and counts the rest', () => {
    expect(clocksText([{ label: 'ILCE-7M3', ms: 0 }])).toBe('No shift');
    expect(clocksText([{ label: 'ILCE-7M3', ms: -3_600_000 }])).toBe('Shift −1:00:00');
    expect(
      clocksText([
        { label: 'ILCE-7M3', ms: -(4 * 3600 + 59 * 60 + 38) * 1000 },
        { label: 'ILCE-7M4', ms: 0 },
        { label: 'iPhone 15 Pro', ms: 0 }
      ])
    ).toBe('ILCE-7M3 −4:59:38 · 2 cameras unchanged');
    expect(
      clocksText([
        { label: 'A', ms: 0 },
        { label: 'B', ms: 0 }
      ])
    ).toBe('No shift');
  });
});
