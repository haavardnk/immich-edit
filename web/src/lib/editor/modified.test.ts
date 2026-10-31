import { AI_DENOISE_STRENGTH, neutralEdits } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import { modifiedDevelopPanels, modifiedTabCount } from './modified';

describe('editor modified state', () => {
  it('maps active operators to their develop panels', () => {
    const edits = neutralEdits();
    edits.basic.exposure_ev = 1;
    edits.color.hsl.bands[2]!.sat = 20;
    edits.effects.grain_amount = 10;

    expect([...modifiedDevelopPanels(edits)]).toEqual(['basic', 'hsl', 'effects']);
    expect(modifiedTabCount(edits, 'develop')).toBe(3);
  });

  it('counts mask and retouch items and geometry as one section', () => {
    const edits = neutralEdits();
    edits.geometry.rotate = 90;
    edits.masks = [{ id: 'mask' } as never];
    edits.retouch = [{ id: 'stroke' } as never, { id: 'stroke-2' } as never];

    expect(modifiedTabCount(edits, 'geometry')).toBe(1);
    expect(modifiedTabCount(edits, 'masks')).toBe(1);
    expect(modifiedTabCount(edits, 'retouch')).toBe(2);
  });

  it('reports no modified sections for neutral edits', () => {
    const edits = neutralEdits();

    expect(modifiedDevelopPanels(edits).size).toBe(0);
    expect(modifiedTabCount(edits, 'develop')).toBe(0);
  });

  it.each([
    ['on at the default strength', true, AI_DENOISE_STRENGTH, true],
    ['on at a custom strength', true, 40, true],
    ['off', false, AI_DENOISE_STRENGTH, false]
  ])('marks Detail modified only while AI denoise is on (%s)', (_name, on, strength, modified) => {
    const edits = neutralEdits();
    edits.detail.ai_denoise = on;
    edits.detail.ai_denoise_strength = strength;

    expect(modifiedDevelopPanels(edits).has('detail')).toBe(modified);
  });

  it('marks Lens Blur modified only while it renders', () => {
    const edits = neutralEdits();
    edits.lens_blur.depth_raster_id = 'depth';
    expect(modifiedDevelopPanels(edits).has('lens-blur')).toBe(false);
    edits.lens_blur.amount = 30;
    expect(modifiedDevelopPanels(edits).has('lens-blur')).toBe(true);
  });
});
