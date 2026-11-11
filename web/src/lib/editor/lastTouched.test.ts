import { neutralEdits } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import { lastTouchedTool } from './lastTouched';

describe('lastTouchedTool', () => {
  it('names the one tool the newest step changed', () => {
    const before = neutralEdits();
    const after = neutralEdits();
    after.color.hsl.bands[2] = { hue: 0, sat: 20, lum: 0 };
    expect(lastTouchedTool({ before, after })).toBe('hsl');
  });

  it('tells tone apart from presence within Basic', () => {
    const before = neutralEdits();
    const after = neutralEdits();
    after.tone.shadows = 30;
    expect(lastTouchedTool({ before, after })).toBe('tone');
    after.tone.shadows = 0;
    after.basic.vibrance = 10;
    expect(lastTouchedTool({ before, after })).toBe('presence');
  });

  it('is unknown without a predecessor or when a step changed several tools', () => {
    const after = neutralEdits();
    after.basic.exposure_ev = 1;
    after.effects.grain_amount = 20;
    expect(lastTouchedTool(null)).toBeNull();
    expect(lastTouchedTool({ before: neutralEdits(), after })).toBeNull();
  });

  it('ignores a change to the depth map alone', () => {
    const after = neutralEdits();
    after.lens_blur.depth_raster_id = 'depth';
    expect(lastTouchedTool({ before: neutralEdits(), after })).toBeNull();
  });
});
