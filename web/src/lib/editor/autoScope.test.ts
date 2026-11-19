import { neutralEdits, type Edits } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import { pickAuto, sectionAuto } from './autoScope';

function suggestion(): Edits {
  const edits = neutralEdits();
  edits.basic.exposure_ev = 0.4;
  edits.basic.brightness = 3;
  edits.basic.contrast = 12;
  edits.basic.clarity = 8;
  edits.basic.vibrance = 15;
  edits.basic.white_balance = { mode: 'absolute', kelvin: 5600, tint: 4 };
  edits.tone = { highlights: -40, shadows: 25, whites: 9, blacks: -11 };
  edits.color.hsl.bands[1] = { hue: 0, sat: 18, lum: 0 };
  return edits;
}

function current(): Edits {
  const edits = neutralEdits();
  edits.basic.clarity = -5;
  edits.color.hsl.bands[4] = { hue: 10, sat: -20, lum: 5 };
  return edits;
}

describe('auto per section', () => {
  it('Tone takes only exposure, brightness, contrast and the tone sliders', () => {
    const next = pickAuto('tone', current(), suggestion());
    expect(next.basic).toMatchObject({ exposure_ev: 0.4, brightness: 3, contrast: 12 });
    expect(next.tone).toEqual({ highlights: -40, shadows: 25, whites: 9, blacks: -11 });
    expect(next.basic.clarity).toBe(-5);
    expect(next.basic.vibrance).toBe(0);
    expect(next.basic.white_balance).toEqual(current().basic.white_balance);
    expect(next.color.hsl).toEqual(current().color.hsl);
  });

  it('the global Auto leaves HSL alone', () => {
    const next = pickAuto('all', current(), suggestion());
    expect(next.basic).toMatchObject({ exposure_ev: 0.4, clarity: 8, vibrance: 15 });
    expect(next.basic.white_balance).toEqual(suggestion().basic.white_balance);
    expect(next.color.hsl).toEqual(current().color.hsl);
  });

  it('offers a section wand only for Tone', () => {
    expect(sectionAuto('tone')).toBe('tone');
    expect(sectionAuto('hsl')).toBeNull();
    expect(sectionAuto('presence')).toBeNull();
  });
});
