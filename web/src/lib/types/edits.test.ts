import { describe, it, expect } from 'vitest';
import {
  neutralEdits,
  originalPreviewEdits,
  resetDevelopEdits,
  editsToManifest,
  manifestToEdits,
  isIdentity,
  isNonGeometryIdentity,
  curvesEditsIsIdentity,
  type Edits
} from './edits';

function roundTrip(e: Edits): Edits {
  return manifestToEdits(editsToManifest(e));
}

describe('neutralEdits identity', () => {
  it('is an identity edit', () => {
    const e = neutralEdits();
    expect(isIdentity(e)).toBe(true);
    expect(isNonGeometryIdentity(e)).toBe(true);
    expect(curvesEditsIsIdentity(e.basic.curves)).toBe(true);
  });

  it('serializes to an empty op set', () => {
    const manifest = editsToManifest(neutralEdits());
    expect(manifest.schema_version).toBe(3);
    expect(Object.keys(manifest.ops)).toHaveLength(0);
  });

  it('defaults camera profile to auto', () => {
    expect(neutralEdits().color.dcp.mode).toBe('auto');
  });

  it('reset preserves camera profile and geometry', () => {
    const e = neutralEdits();
    e.basic.exposure_ev = 2;
    e.color.dcp.mode = 'profile';
    e.color.dcp.profile_id = 'sony';
    e.geometry.rotate = 90;
    const reset = resetDevelopEdits(e);
    expect(reset.basic.exposure_ev).toBe(0);
    expect(reset.color.dcp).toEqual(e.color.dcp);
    expect(reset.geometry).toEqual(e.geometry);
  });

  it('camera profile alone does not enable develop reset', () => {
    const e = neutralEdits();
    e.color.dcp.mode = 'off';
    expect(isNonGeometryIdentity(e)).toBe(true);
    expect(isIdentity(e)).toBe(false);
    e.color.dcp.mode = 'profile';
    e.color.dcp.profile_id = 'sony';
    expect(isNonGeometryIdentity(e)).toBe(true);
    expect(isIdentity(e)).toBe(false);
  });

  it('original preview keeps rendering baselines only', () => {
    const e = neutralEdits();
    e.basic.exposure_ev = 1;
    e.color.lut_3d.lut_id = 'lut';
    e.color.dcp.mode = 'profile';
    e.color.dcp.profile_id = 'sony';
    e.geometry.rotate = 90;
    e.lens.profile_enabled = true;
    const original = originalPreviewEdits(e);
    expect(original.basic.exposure_ev).toBe(0);
    expect(original.color.lut_3d.lut_id).toBeNull();
    expect(original.color.dcp).toEqual(e.color.dcp);
    expect(original.geometry).toEqual(e.geometry);
    expect(original.lens).toEqual(e.lens);
  });

  it('treats mask-only edits as non-identity', () => {
    const e = neutralEdits();
    e.masks = [
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
    expect(isIdentity(e)).toBe(false);
    expect(isNonGeometryIdentity(e)).toBe(false);
  });
});

describe('editsToManifest / manifestToEdits round-trip', () => {
  it('preserves basic adjustments', () => {
    const e = neutralEdits();
    e.basic.exposure_ev = 1.25;
    e.basic.brightness = 10;
    e.basic.contrast = -15;
    e.basic.saturation = 8;
    e.basic.vibrance = -4;
    e.basic.wb_temp = 200;
    e.basic.wb_tint = -12;
    e.basic.texture = 30;
    e.basic.clarity = 20;
    e.basic.dehaze = 5;
    const back = roundTrip(e);
    expect(back.basic).toEqual(e.basic);
  });

  it('preserves tone regions', () => {
    const e = neutralEdits();
    e.tone = { highlights: -40, shadows: 30, blacks: -10, whites: 15 };
    const back = roundTrip(e);
    expect(back.tone).toEqual(e.tone);
  });

  it('preserves lut selection and amount', () => {
    const e = neutralEdits();
    e.color.lut_3d = { lut_id: 'abc123', amount: 65 };
    const back = roundTrip(e);
    expect(back.color.lut_3d).toEqual(e.color.lut_3d);
  });

  it('omits inactive lut from manifest', () => {
    const e = neutralEdits();
    e.color.lut_3d = { lut_id: null, amount: 100 };
    expect(editsToManifest(e).ops.lut_3d).toBeUndefined();
    e.color.lut_3d = { lut_id: 'x', amount: 0 };
    expect(editsToManifest(e).ops.lut_3d).toBeUndefined();
  });

  it('preserves dcp profile selection and toggles', () => {
    const e = neutralEdits();
    e.color.dcp = {
      mode: 'profile',
      profile_id: 'abc123',
      illuminant: 'second',
      use_tone_curve: true,
      use_base_table: false,
      use_look_table: false,
      use_baseline_exposure: true
    };
    const back = roundTrip(e);
    expect(back.color.dcp).toEqual(e.color.dcp);
  });

  it('preserves explicit dcp off mode', () => {
    const e = neutralEdits();
    e.color.dcp.mode = 'off';
    const back = roundTrip(e);
    expect(back.color.dcp.mode).toBe('off');
  });

  it('omits default auto dcp and persists explicit off', () => {
    const e = neutralEdits();
    expect(editsToManifest(e).ops.dcp_hue_sat).toBeUndefined();
    e.color.dcp.mode = 'off';
    expect(editsToManifest(e).ops.dcp_hue_sat).toBeDefined();
  });

  it('preserves detail and effects', () => {
    const e = neutralEdits();
    e.detail.sharpen_amount = 40;
    e.detail.sharpen_radius = 1.2;
    e.detail.luma_nr_amount = 25;
    e.detail.color_nr_amount = 15;
    e.effects.vignette_amount = -30;
    e.effects.grain_amount = 20;
    const back = roundTrip(e);
    expect(back.detail).toEqual(e.detail);
    expect(back.effects).toEqual(e.effects);
  });

  it('preserves every flat op field', () => {
    const e = neutralEdits();
    e.basic.exposure_ev = 1.5;
    e.basic.brightness = 11;
    e.basic.contrast = 12;
    e.basic.saturation = 13;
    e.basic.vibrance = 14;
    e.basic.texture = 15;
    e.basic.clarity = 16;
    e.basic.dehaze = 17;
    e.basic.wb_temp = 18;
    e.basic.wb_tint = 19;
    e.tone.highlights = 21;
    e.tone.shadows = 22;
    e.tone.blacks = 23;
    e.tone.whites = 24;
    e.detail.sharpen_amount = 31;
    e.detail.sharpen_radius = 3.2;
    e.detail.sharpen_detail = 33;
    e.detail.sharpen_masking = 34;
    e.detail.luma_nr_amount = 35;
    e.detail.luma_nr_detail = 36;
    e.detail.luma_nr_contrast = 37;
    e.detail.color_nr_amount = 38;
    e.detail.color_nr_detail = 39;
    e.detail.color_nr_smoothness = 40;
    e.effects.vignette_amount = 41;
    e.effects.vignette_midpoint = 42;
    e.effects.vignette_feather = 43;
    e.effects.vignette_roundness = 44;
    e.effects.grain_amount = 45;
    e.effects.grain_size = 46;
    e.effects.grain_roughness = 47;
    e.lens.profile_enabled = true;
    e.lens.ca_enabled = true;
    e.lens.constrain_crop = true;
    e.lens.distortion_amount = 51;
    e.lens.vignette_amount = 52;
    e.lens.k1 = 53;
    e.lens.k2 = 54;
    e.lens.k3 = 55;
    e.lens.vk1 = 56;
    e.lens.vk2 = 57;
    e.lens.vk3 = 58;
    e.lens.ca_red_scale_x10000 = 59;
    e.lens.ca_blue_scale_x10000 = 60;
    const back = roundTrip(e);
    expect(back.basic.curves).toEqual(e.basic.curves);
    expect(back.tone).toEqual(e.tone);
    expect(back.detail).toEqual(e.detail);
    expect(back.effects).toEqual(e.effects);
    expect(back.lens).toEqual(e.lens);
    expect(back.basic).toEqual(e.basic);
  });

  it('preserves a custom composite curve', () => {
    const e = neutralEdits();
    e.basic.curves.composite = [
      { x: 0, y: 0.05 },
      { x: 0.5, y: 0.6 },
      { x: 1, y: 0.95 }
    ];
    const back = roundTrip(e);
    expect(back.basic.curves.composite).toEqual(e.basic.curves.composite);
  });

  it('preserves geometry crop and rotation', () => {
    const e = neutralEdits();
    e.geometry.rotate = 90;
    e.geometry.rotate_angle = 3.5;
    e.geometry.flip_h = true;
    e.geometry.crop = { x: 0.1, y: 0.1, w: 0.8, h: 0.7 };
    e.geometry.aspect = { kind: 'ratio', num: 16, den: 9 };
    const back = roundTrip(e);
    expect(back.geometry.rotate).toBe(90);
    expect(back.geometry.rotate_angle).toBeCloseTo(3.5, 6);
    expect(back.geometry.flip_h).toBe(true);
    expect(back.geometry.crop).toEqual(e.geometry.crop);
    expect(back.geometry.aspect).toEqual(e.geometry.aspect);
  });

  it('decodes a legacy single-curve points payload', () => {
    const edits = manifestToEdits({
      schema_version: 3,
      ops: { curves: { points: [[0, 0], [0.5, 0.7], [1, 1]] } }
    });
    expect(edits.basic.curves.composite).toEqual([
      { x: 0, y: 0 },
      { x: 0.5, y: 0.7 },
      { x: 1, y: 1 }
    ]);
  });

  it('accepts the legacy highlights_shadows alias for tone regions', () => {
    const edits = manifestToEdits({
      schema_version: 3,
      ops: { highlights_shadows: { highlights: -10, shadows: 20 } }
    });
    expect(edits.tone.highlights).toBe(-10);
    expect(edits.tone.shadows).toBe(20);
  });

  it('preserves luma and color range masks', () => {
    const e = neutralEdits();
    e.masks = [
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
            kind: { kind: 'luma_range', min: 0.2, max: 0.8, softness: 0.1 },
            source: 'manual'
          },
          {
            id: 'color',
            enabled: true,
            mode: 'intersect',
            invert: false,
            kind: {
              kind: 'color_range',
              sample_rgb: [0.2, 0.5, 0.9],
              tolerance: 0.12,
              softness: 0.04
            },
            source: 'manual'
          }
        ],
        edits: { exposure_ev: 0.5 }
      }
    ];
    expect(roundTrip(e).masks).toEqual(e.masks);
  });

  it('round trips polygon components', () => {
    const e = neutralEdits();
    e.masks = [
      {
        id: 'layer',
        name: 'Shapes',
        enabled: true,
        color: '#ff3b30',
        amount: 1,
        invert: false,
        components: [
          {
            id: 'poly',
            enabled: true,
            mode: 'add',
            invert: false,
            kind: {
              kind: 'polygon',
              points: [
                { x: 0.1, y: 0.2 },
                { x: 0.8, y: 0.15 },
                { x: 0.5, y: 0.9 }
              ],
              feather: 0.08
            },
            source: 'manual'
          }
        ],
        edits: {}
      }
    ];
    expect(roundTrip(e).masks).toEqual(e.masks);
  });

  it('preserves generated mask provenance', () => {
    const e = neutralEdits();
    e.masks = [
      {
        id: 'layer',
        name: 'Subject',
        enabled: true,
        color: '#ff3b30',
        amount: 1,
        invert: false,
        components: [
          {
            id: 'gen',
            enabled: true,
            mode: 'add',
            invert: false,
            kind: { kind: 'brush', raster_id: 'baked' },
            source: 'generated',
            generated: {
              model_id: 'ormbg',
              kind: 'subject',
              prob_raster_id: 'prob',
              grow: -2,
              feather: 4
            }
          }
        ],
        edits: { exposure_ev: 0.5 }
      }
    ];
    expect(roundTrip(e).masks).toEqual(e.masks);
  });

  it('preserves click points on a refinable mask', () => {
    const e = neutralEdits();
    e.masks = [
      {
        id: 'layer',
        name: 'Click',
        enabled: true,
        color: '#ff3b30',
        amount: 1,
        invert: false,
        components: [
          {
            id: 'gen',
            enabled: true,
            mode: 'add',
            invert: false,
            kind: { kind: 'brush', raster_id: 'baked' },
            source: 'generated',
            generated: {
              model_id: 'mobilesam',
              kind: 'click',
              prob_raster_id: 'prob',
              grow: 0,
              feather: 0,
              points: [
                { x: 0.25, y: 0.5, positive: true },
                { x: 0.8, y: 0.1, positive: false }
              ]
            }
          }
        ],
        edits: { exposure_ev: 0.5 }
      }
    ];
    expect(roundTrip(e).masks).toEqual(e.masks);
  });

  it('drops malformed color range samples', () => {
    const edits = manifestToEdits({
      schema_version: 3,
      ops: {
        masks: {
          layers: [
            {
              id: 'layer',
              components: [
                {
                  id: 'color',
                  kind: {
                    kind: 'color_range',
                    sample_rgb: [0.2, 0.5],
                    tolerance: 0.1,
                    softness: 0.05
                  }
                }
              ],
              edits: { exposure_ev: 0.5 }
            }
          ]
        }
      }
    });
    expect(edits.masks).toEqual([]);
  });
});
