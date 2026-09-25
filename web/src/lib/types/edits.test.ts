import { describe, it, expect } from 'vitest';
import {
  neutralEdits,
  isIdentity,
  isNonGeometryIdentity,
  curvesEditsIsIdentity,
  neutralSharpenAmount,
  effectiveLens,
  neutraliseSection,
  originalPreviewEdits,
  resetDevelopEdits,
  type DevelopSection,
  type Edits
} from './edits';

describe('neutralEdits identity', () => {
  it('is an identity edit', () => {
    const edits = neutralEdits();
    expect(isIdentity(edits)).toBe(true);
    expect(isNonGeometryIdentity(edits)).toBe(true);
    expect(curvesEditsIsIdentity(edits.basic.curves)).toBe(true);
  });

  it('defaults camera profile to auto', () => {
    expect(neutralEdits().color.dcp.mode).toBe('auto');
  });

  it('reset preserves camera profile and geometry', () => {
    const edits = neutralEdits();
    edits.basic.exposure_ev = 2;
    edits.color.dcp.mode = 'profile';
    edits.color.dcp.profile_id = 'sony';
    edits.geometry.rotate = 90;
    const reset = resetDevelopEdits(edits);
    expect(reset.basic.exposure_ev).toBe(0);
    expect(reset.color.dcp).toEqual(edits.color.dcp);
    expect(reset.geometry).toEqual(edits.geometry);
  });

  it('camera profile alone does not enable develop reset', () => {
    const edits = neutralEdits();
    edits.color.dcp.mode = 'off';
    expect(isNonGeometryIdentity(edits)).toBe(true);
    expect(isIdentity(edits)).toBe(false);
    edits.color.dcp.mode = 'profile';
    edits.color.dcp.profile_id = 'sony';
    expect(isNonGeometryIdentity(edits)).toBe(true);
    expect(isIdentity(edits)).toBe(false);
  });

  it('original preview keeps rendering baselines only', () => {
    const edits = neutralEdits();
    edits.basic.exposure_ev = 1;
    edits.color.lut_3d.lut_id = 'lut';
    edits.color.dcp.mode = 'profile';
    edits.color.dcp.profile_id = 'sony';
    edits.geometry.rotate = 90;
    edits.lens.profile_enabled = true;
    const original = originalPreviewEdits(edits);
    expect(original.basic.exposure_ev).toBe(0);
    expect(original.color.lut_3d.lut_id).toBeNull();
    expect(original.color.dcp).toEqual(edits.color.dcp);
    expect(original.geometry).toEqual(edits.geometry);
    expect(original.lens).toEqual(edits.lens);
  });

  it('treats mask-only edits as non-identity', () => {
    const edits = neutralEdits();
    edits.masks = [
      {
        id: 'layer',
        name: 'Range',
        enabled: true,
        color: '#ff3b30',
        amount: 1,
        invert: false,
        components: [
          {
            id: 'luma',
            enabled: true,
            mode: 'add',
            invert: false,
            kind: { kind: 'luma_range', min: 0.25, max: 0.75, softness: 0.1 },
            source: 'manual'
          }
        ],
        edits: {}
      }
    ];
    expect(isIdentity(edits)).toBe(false);
    expect(isNonGeometryIdentity(edits)).toBe(false);
  });

  it('treats retouch-only edits as non-identity', () => {
    const edits = neutralEdits();
    edits.retouch = [
      {
        id: 'spot',
        mode: 'heal',
        points: [{ x: 0.4, y: 0.5 }],
        radius: 0.05,
        hardness: 0.5,
        opacity: 1,
        source: { x: 0.6, y: 0.5 },
        enabled: true
      }
    ];
    expect(isIdentity(edits)).toBe(false);
    expect(isNonGeometryIdentity(edits)).toBe(false);
  });
});

const SECTION_FIELDS: Record<DevelopSection, string[]> = {
  white_balance: ['basic.wb_temp', 'basic.wb_tint'],
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
    'detail.color_nr_smoothness'
  ],
  vignette: [
    'effects.vignette_amount',
    'effects.vignette_midpoint',
    'effects.vignette_feather',
    'effects.vignette_roundness'
  ],
  grain: ['effects.grain_amount', 'effects.grain_size', 'effects.grain_roughness'],
  lens: ['lens.profile_enabled', 'lens.k1'],
  lut: ['color.lut_3d.lut_id', 'color.lut_3d.amount'],
  bw: ['color.bw.enabled', 'color.bw.mix.green', 'color.bw.shadows.sat', 'color.bw.balance']
};

function read(edits: Edits, path: string): unknown {
  return path
    .split('.')
    .reduce<unknown>((node, key) => (node as Record<string, unknown>)[key], edits);
}

function graded(): Edits {
  const edits = neutralEdits();
  edits.basic.wb_temp = 30;
  edits.basic.wb_tint = -20;
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
  edits.color.bw.enabled = true;
  edits.color.bw.mix.green = 40;
  edits.color.bw.shadows.sat = 25;
  edits.color.bw.balance = -30;
  edits.geometry.rotate = 90;
  return edits;
}

describe('neutraliseSection', () => {
  const sections = Object.keys(SECTION_FIELDS) as DevelopSection[];

  it.each(sections)('%s drops its own fields and keeps the rest', (section) => {
    const edits = graded();
    const neutral = neutralEdits();
    const bypassed = neutraliseSection(edits, section);

    for (const field of SECTION_FIELDS[section]) {
      expect(read(bypassed, field)).toEqual(read(neutral, field));
    }
    for (const other of sections.filter((name) => name !== section)) {
      for (const field of SECTION_FIELDS[other]) {
        expect(read(bypassed, field)).toEqual(read(edits, field));
      }
    }
    expect(bypassed.color.hsl).toEqual(edits.color.hsl);
    expect(bypassed.geometry).toEqual(edits.geometry);
    expect(bypassed.color.dcp).toEqual(edits.color.dcp);
  });

  it('leaves the source edits untouched', () => {
    const edits = graded();
    neutraliseSection(edits, 'tone');
    expect(edits.basic.exposure_ev).toBe(1);
    expect(edits.tone.shadows).toBe(40);
  });
});

describe('edit model defaults', () => {
  it('resolves the unset sharpen amount per frame type', () => {
    expect(neutralSharpenAmount(true)).toBe(40);
    expect(neutralSharpenAmount(false)).toBe(0);
  });

  it('resolves the auto lens baseline only when unset', () => {
    const profile = { k1: -0.1, k2: 0, k3: 0, vk1: -0.3, vk2: 0, vk3: 0 };
    const auto = effectiveLens(neutralEdits().lens, profile);
    expect(auto.profile_enabled).toBe(true);
    expect(auto.constrain_crop).toBe(true);
    expect(auto.k1).toBe(-0.1);

    for (const explicit of [true, false]) {
      const lens = { ...neutralEdits().lens, profile_enabled: explicit };
      expect(effectiveLens(lens, profile)).toEqual(lens);
    }
    expect(effectiveLens(neutralEdits().lens, null).profile_enabled).toBeNull();
  });
});
