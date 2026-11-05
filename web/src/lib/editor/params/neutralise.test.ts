import { neutralEdits, type DevelopSection, type Edits } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import { neutraliseSection } from './neutralise';

const SECTION_FIELDS: Record<DevelopSection, string[]> = {
  dcp: ['color.dcp'],
  white_balance: ['basic.white_balance'],
  curves: ['basic.curves'],
  tone: [
    'basic.exposure_ev',
    'basic.brightness',
    'basic.contrast',
    'tone.highlights',
    'tone.shadows',
    'tone.whites',
    'tone.blacks'
  ],
  presence: [
    'basic.texture',
    'basic.clarity',
    'basic.dehaze',
    'basic.vibrance',
    'basic.saturation'
  ],
  sharpening: [
    'detail.capture_sharpen',
    'detail.capture_corner_boost',
    'detail.sharpen_amount',
    'detail.sharpen_radius',
    'detail.sharpen_detail',
    'detail.sharpen_masking'
  ],
  noise_reduction: [
    'detail.luma_nr_amount',
    'detail.luma_nr_detail',
    'detail.luma_nr_contrast',
    'detail.color_nr_amount',
    'detail.color_nr_detail',
    'detail.color_nr_smoothness',
    'detail.ai_denoise',
    'detail.ai_denoise_strength',
    'detail.ai_denoise_detail'
  ],
  vignette: [
    'effects.vignette_amount',
    'effects.vignette_midpoint',
    'effects.vignette_feather',
    'effects.vignette_roundness'
  ],
  grain: ['effects.grain_amount', 'effects.grain_size', 'effects.grain_roughness'],
  lens: ['lens.profile_enabled', 'lens.k1'],
  hsl: ['color.hsl.bands'],
  color_grading: ['color.color_grade.midtones', 'color.color_grade.balance'],
  lut: ['color.lut_3d.lut_id', 'color.lut_3d.amount'],
  bw: ['color.bw.enabled', 'color.bw.mix.green', 'color.bw.shadows.sat', 'color.bw.balance'],
  lens_blur: [
    'lens_blur.amount',
    'lens_blur.focus_min',
    'lens_blur.focus_max',
    'lens_blur.shape',
    'lens_blur.boost'
  ]
};

function read(edits: Edits, path: string): unknown {
  return path
    .split('.')
    .reduce<unknown>((node, key) => (node as Record<string, unknown>)[key], edits);
}

function graded(): Edits {
  const edits = neutralEdits();
  edits.basic.white_balance = { mode: 'absolute', kelvin: 4200, tint: -20 };
  edits.basic.exposure_ev = 1;
  edits.basic.brightness = 10;
  edits.basic.contrast = 20;
  edits.tone.highlights = -30;
  edits.tone.shadows = 40;
  edits.tone.whites = 5;
  edits.tone.blacks = -5;
  edits.basic.texture = 15;
  edits.basic.clarity = 25;
  edits.basic.dehaze = 35;
  edits.basic.vibrance = 45;
  edits.basic.saturation = 55;
  edits.detail.capture_sharpen = false;
  edits.detail.capture_corner_boost = 30;
  edits.detail.sharpen_amount = 80;
  edits.detail.sharpen_radius = 2;
  edits.detail.sharpen_detail = 60;
  edits.detail.sharpen_masking = 40;
  edits.detail.luma_nr_amount = 25;
  edits.detail.luma_nr_detail = 60;
  edits.detail.luma_nr_contrast = 30;
  edits.detail.color_nr_amount = 35;
  edits.detail.color_nr_detail = 70;
  edits.detail.color_nr_smoothness = 20;
  edits.detail.ai_denoise = true;
  edits.detail.ai_denoise_strength = 40;
  edits.color.dcp.mode = 'flat';
  edits.basic.curves.composite = [
    { x: 0, y: 0 },
    { x: 0.5, y: 0.6 },
    { x: 1, y: 1 }
  ];
  edits.effects.vignette_amount = -40;
  edits.effects.vignette_midpoint = 60;
  edits.effects.vignette_feather = 30;
  edits.effects.vignette_roundness = 20;
  edits.effects.grain_amount = 30;
  edits.effects.grain_size = 40;
  edits.effects.grain_roughness = 60;
  edits.lens.profile_enabled = true;
  edits.lens.k1 = 0.02;
  edits.color.lut_3d.lut_id = 'lut';
  edits.color.lut_3d.amount = 60;
  edits.color.hsl.bands = edits.color.hsl.bands.map((band) => ({ ...band, hue: 10 }));
  edits.color.color_grade.midtones = { hue: 200, sat: 30, lum: 10 };
  edits.color.color_grade.balance = 20;
  edits.color.bw.enabled = true;
  edits.color.bw.mix.green = 40;
  edits.color.bw.shadows.sat = 25;
  edits.color.bw.balance = -30;
  edits.lens_blur = {
    amount: 60,
    focus_min: 0.1,
    focus_max: 0.5,
    shape: 'hexagon',
    boost: 40,
    depth_raster_id: 'depth'
  };
  edits.geometry.rotate = 90;
  return edits;
}

describe('neutraliseSection', () => {
  const sections = Object.keys(SECTION_FIELDS) as DevelopSection[];

  it.each(sections)('%s drops its own fields and keeps the rest', (section) => {
    const edits = graded();
    const neutral = neutralEdits();
    const bypassed = neutraliseSection(edits, section, false);

    for (const field of SECTION_FIELDS[section]) {
      expect(read(bypassed, field)).toEqual(read(neutral, field));
    }
    for (const other of sections.filter((name) => name !== section)) {
      for (const field of SECTION_FIELDS[other]) {
        expect(read(bypassed, field)).toEqual(read(edits, field));
      }
    }
    expect(bypassed.geometry).toEqual(edits.geometry);
    expect(bypassed.lens_blur.depth_raster_id).toBe('depth');
  });

  it.each(sections)('leaves the source edits untouched (%s)', (section) => {
    const edits = graded();
    neutraliseSection(edits, section, false);
    expect(edits).toStrictEqual(graded());
  });

  it.each([false, true])('stores the neutral sharpen amount as unset (raw %s)', (isRaw) => {
    const edits = graded();
    edits.detail.sharpen_amount = 90;
    expect(neutraliseSection(edits, 'sharpening', isRaw).detail.sharpen_amount).toBeNull();
  });

  it('shares the geometry, masks and retouch it does not own', () => {
    const edits = graded();
    const bypassed = neutraliseSection(edits, 'bw', false);
    expect(bypassed.geometry).toBe(edits.geometry);
    expect(bypassed.masks).toBe(edits.masks);
    expect(bypassed.retouch).toBe(edits.retouch);
  });
});
