import { editsToManifest } from '#lib/edits/manifest.ts';
import { PARAMS, TOUCH_GROUPS } from '#lib/editor/params/catalog.ts';
import { everyDevelopOpModified } from '#lib/testing/developEdits.ts';
import { AI_DENOISE_STRENGTH, neutralEdits, type Edits } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import {
  DEVELOP_PANEL_OPS,
  TOUCH_TOOL_OPS,
  modifiedDevelopPanels,
  modifiedTabCount,
  modifiedTouchGroups
} from './modified';

function everyDevelopOpActive(): Edits {
  const edits = everyDevelopOpModified();
  for (const param of PARAMS) {
    param.set(edits, param.defaultValue === param.max ? param.min : param.max, true);
  }
  edits.basic.white_balance = { mode: 'relative', temp: 10, tint: 0 };
  edits.detail.ai_denoise = true;
  edits.color.lut_3d.amount = 100;
  return edits;
}

const developOps = Object.keys(editsToManifest(everyDevelopOpActive()).ops)
  .filter((op) => op !== 'transform')
  .sort();

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

  it.each([
    ['desktop panels', DEVELOP_PANEL_OPS],
    ['touch groups', TOUCH_TOOL_OPS]
  ] as const)('assigns every develop operator to exactly one of the %s', (_name, table) => {
    const ops = Object.values<readonly string[]>(table).flat();

    expect([...ops].sort()).toEqual(developOps);
    expect(new Set(ops).size).toBe(ops.length);
  });

  it('flags every touch group when every operator is modified', () => {
    expect([...modifiedTouchGroups(everyDevelopOpModified())]).toEqual(
      TOUCH_GROUPS.map((group) => group.id)
    );
  });

  it('maps active operators to their touch groups', () => {
    const edits = neutralEdits();
    edits.basic.white_balance = { mode: 'relative', temp: 10, tint: 0 };
    edits.basic.texture = 20;

    expect([...modifiedTouchGroups(edits)]).toEqual(['color', 'effects']);
  });

  it('marks Lens Blur modified only while it renders', () => {
    const edits = neutralEdits();
    edits.lens_blur.depth_raster_id = 'depth';
    expect(modifiedDevelopPanels(edits).has('lens-blur')).toBe(false);
    expect(modifiedTouchGroups(edits).has('effects')).toBe(false);
    edits.lens_blur.amount = 30;
    expect(modifiedDevelopPanels(edits).has('lens-blur')).toBe(true);
    expect(modifiedTouchGroups(edits).has('effects')).toBe(true);
  });
});
