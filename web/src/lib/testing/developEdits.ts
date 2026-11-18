import { neutralEdits, type Edits } from '#lib/types/edits.ts';

export function everyDevelopOpModified(): Edits {
  const edits = neutralEdits();
  edits.color.dcp.mode = 'flat';
  edits.basic.exposure_ev = 1;
  edits.tone.shadows = 20;
  edits.basic.curves.composite = [
    { x: 0, y: 0 },
    { x: 0.5, y: 0.6 },
    { x: 1, y: 1 }
  ];
  edits.color.hsl.bands = edits.color.hsl.bands.map((band, index) =>
    index === 2 ? { ...band, sat: 20 } : band
  );
  edits.color.color_grade.shadows.sat = 30;
  edits.color.bw.enabled = true;
  edits.color.lut_3d.lut_id = 'lut-1';
  edits.detail.luma_nr_amount = 25;
  edits.lens.k1 = 0.1;
  edits.lens_blur.amount = 40;
  edits.lens_blur.depth_raster_id = 'depth';
  edits.effects.grain_amount = 10;
  edits.geometry.rotate = 90;
  return edits;
}
