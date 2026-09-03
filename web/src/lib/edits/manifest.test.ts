import { describe, expect, it } from 'vitest';
import { isIdentity, neutralEdits, type Edits } from '$lib/types/edits';
import { editsToManifest, manifestToEdits } from './manifest';

function roundTrip(edits: Edits): Edits {
  return manifestToEdits(editsToManifest(edits));
}

describe('manifest codec', () => {
  it('serializes neutral edits to an empty op set', () => {
    const manifest = editsToManifest(neutralEdits());
    expect(manifest.schema_version).toBe(4);
    expect(Object.keys(manifest.ops)).toHaveLength(0);
  });

  it('preserves basic adjustments', () => {
    const edits = neutralEdits();
    edits.basic.exposure_ev = 1.25;
    edits.basic.brightness = 10;
    edits.basic.contrast = -15;
    edits.basic.saturation = 8;
    edits.basic.vibrance = -4;
    edits.basic.wb_temp = 200;
    edits.basic.wb_tint = -12;
    edits.basic.texture = 30;
    edits.basic.clarity = 20;
    edits.basic.dehaze = 5;
    expect(roundTrip(edits).basic).toEqual(edits.basic);
  });

  it('preserves tone regions', () => {
    const edits = neutralEdits();
    edits.tone = { highlights: -40, shadows: 30, blacks: -10, whites: 15 };
    expect(roundTrip(edits).tone).toEqual(edits.tone);
  });

  it('preserves lut selection and amount', () => {
    const edits = neutralEdits();
    edits.color.lut_3d = { lut_id: 'abc123', amount: 65 };
    expect(roundTrip(edits).color.lut_3d).toEqual(edits.color.lut_3d);
  });

  it('omits inactive lut from manifest', () => {
    const edits = neutralEdits();
    edits.color.lut_3d = { lut_id: null, amount: 100 };
    expect(editsToManifest(edits).ops.lut_3d).toBeUndefined();
    edits.color.lut_3d = { lut_id: 'x', amount: 0 };
    expect(editsToManifest(edits).ops.lut_3d).toBeUndefined();
  });

  it.each([null, 0, 40, 90])('preserves sharpen amount %s', (amount) => {
    const edits = neutralEdits();
    edits.detail.sharpen_amount = amount;
    expect(roundTrip(edits).detail.sharpen_amount).toBe(amount);
  });

  it('leaves an unset sharpen amount out of the manifest', () => {
    const edits = neutralEdits();
    expect(edits.detail.sharpen_amount).toBeNull();
    expect(editsToManifest(edits).ops.sharpen).toBeUndefined();
    edits.detail.sharpen_amount = 0;
    expect(editsToManifest(edits).ops.sharpen).toBeDefined();
    expect(isIdentity(edits)).toBe(false);
  });

  it('preserves dcp profile selection and toggles', () => {
    const edits = neutralEdits();
    edits.color.dcp = {
      mode: 'profile',
      profile_id: 'abc123',
      illuminant: 'second',
      use_tone_curve: true,
      use_base_table: false,
      use_look_table: false,
      use_baseline_exposure: false
    };
    expect(roundTrip(edits).color.dcp).toEqual(edits.color.dcp);
  });

  it('preserves explicit dcp off mode', () => {
    const edits = neutralEdits();
    edits.color.dcp.mode = 'off';
    expect(roundTrip(edits).color.dcp.mode).toBe('off');
  });

  it('omits default auto dcp and persists explicit off', () => {
    const edits = neutralEdits();
    expect(editsToManifest(edits).ops.dcp_hue_sat).toBeUndefined();
    edits.color.dcp.mode = 'off';
    expect(editsToManifest(edits).ops.dcp_hue_sat).toBeDefined();
  });

  it('preserves detail and effects', () => {
    const edits = neutralEdits();
    edits.detail.sharpen_amount = 40;
    edits.detail.sharpen_radius = 1.2;
    edits.detail.luma_nr_amount = 25;
    edits.detail.color_nr_amount = 15;
    edits.effects.vignette_amount = -30;
    edits.effects.grain_amount = 20;
    const decoded = roundTrip(edits);
    expect(decoded.detail).toEqual(edits.detail);
    expect(decoded.effects).toEqual(edits.effects);
  });

  it('only records capture sharpening when it is turned off', () => {
    const edits = neutralEdits();
    expect(editsToManifest(edits).ops.capture_sharpen).toBeUndefined();
    edits.detail.capture_sharpen = false;
    expect(editsToManifest(edits).ops.capture_sharpen).toEqual({ enabled: false });
    expect(roundTrip(edits).detail.capture_sharpen).toBe(false);
  });

  it('preserves every flat op field', () => {
    const edits = neutralEdits();
    edits.basic.exposure_ev = 1.5;
    edits.basic.brightness = 11;
    edits.basic.contrast = 12;
    edits.basic.saturation = 13;
    edits.basic.vibrance = 14;
    edits.basic.texture = 15;
    edits.basic.clarity = 16;
    edits.basic.dehaze = 17;
    edits.basic.wb_temp = 18;
    edits.basic.wb_tint = 19;
    edits.tone.highlights = 21;
    edits.tone.shadows = 22;
    edits.tone.blacks = 23;
    edits.tone.whites = 24;
    edits.detail.sharpen_amount = 31;
    edits.detail.sharpen_radius = 3.2;
    edits.detail.sharpen_detail = 33;
    edits.detail.sharpen_masking = 34;
    edits.detail.luma_nr_amount = 35;
    edits.detail.luma_nr_detail = 36;
    edits.detail.luma_nr_contrast = 37;
    edits.detail.color_nr_amount = 38;
    edits.detail.color_nr_detail = 39;
    edits.detail.color_nr_smoothness = 40;
    edits.effects.vignette_amount = 41;
    edits.effects.vignette_midpoint = 42;
    edits.effects.vignette_feather = 43;
    edits.effects.vignette_roundness = 44;
    edits.effects.grain_amount = 45;
    edits.effects.grain_size = 46;
    edits.effects.grain_roughness = 47;
    edits.lens.profile_enabled = true;
    edits.lens.ca_enabled = true;
    edits.lens.constrain_crop = true;
    edits.lens.distortion_amount = 51;
    edits.lens.vignette_amount = 52;
    edits.lens.k1 = 53;
    edits.lens.k2 = 54;
    edits.lens.k3 = 55;
    edits.lens.vk1 = 56;
    edits.lens.vk2 = 57;
    edits.lens.vk3 = 58;
    edits.lens.ca_red_scale_x10000 = 59;
    edits.lens.ca_blue_scale_x10000 = 60;
    const decoded = roundTrip(edits);
    expect(decoded.basic.curves).toEqual(edits.basic.curves);
    expect(decoded.tone).toEqual(edits.tone);
    expect(decoded.detail).toEqual(edits.detail);
    expect(decoded.effects).toEqual(edits.effects);
    expect(decoded.lens).toEqual(edits.lens);
    expect(decoded.basic).toEqual(edits.basic);
  });

  it('preserves a custom composite curve', () => {
    const edits = neutralEdits();
    edits.basic.curves.composite = [
      { x: 0, y: 0.05 },
      { x: 0.5, y: 0.6 },
      { x: 1, y: 0.95 }
    ];
    expect(roundTrip(edits).basic.curves.composite).toEqual(edits.basic.curves.composite);
  });

  it('preserves geometry crop and rotation', () => {
    const edits = neutralEdits();
    edits.geometry.rotate = 90;
    edits.geometry.rotate_angle = 3.5;
    edits.geometry.flip_h = true;
    edits.geometry.crop = { x: 0.1, y: 0.1, w: 0.8, h: 0.7 };
    edits.geometry.aspect = { kind: 'ratio', num: 16, den: 9 };
    const decoded = roundTrip(edits);
    expect(decoded.geometry.rotate).toBe(90);
    expect(decoded.geometry.rotate_angle).toBeCloseTo(3.5, 6);
    expect(decoded.geometry.flip_h).toBe(true);
    expect(decoded.geometry.crop).toEqual(edits.geometry.crop);
    expect(decoded.geometry.aspect).toEqual(edits.geometry.aspect);
  });

  it('preserves retouch strokes', () => {
    const edits = neutralEdits();
    edits.retouch = [
      {
        id: 'spot-1',
        mode: 'clone',
        points: [
          { x: 0.3, y: 0.35 },
          { x: 0.32, y: 0.37 }
        ],
        radius: 0.08,
        hardness: 0.4,
        opacity: 0.9,
        source: { x: 0.6, y: 0.55 },
        enabled: false
      }
    ];
    expect(roundTrip(edits).retouch).toEqual(edits.retouch);
  });

  it('caps retouch stroke points and drops pointless strokes', () => {
    const many = Array.from({ length: 400 }, (_, index) => ({ x: index / 400, y: 0.5 }));
    const edits = manifestToEdits({
      schema_version: 3,
      ops: {
        retouch: {
          strokes: [
            { id: 'a', mode: 'heal', points: many, radius: 0.05, source: { x: 0.8, y: 0.5 } },
            { id: 'b', mode: 'heal', points: [], radius: 0.05, source: { x: 0.8, y: 0.5 } },
            { id: 'c', mode: 'heal', points: [{ x: 0.1, y: 0.1 }], radius: 0.05 }
          ]
        }
      }
    });
    expect(edits.retouch).toHaveLength(1);
    expect(edits.retouch[0]?.points).toHaveLength(256);
    expect(edits.retouch[0]?.opacity).toBe(1);
    expect(edits.retouch[0]?.enabled).toBe(true);
  });

  it('decodes a legacy single-curve points payload', () => {
    const edits = manifestToEdits({
      schema_version: 3,
      ops: {
        curves: {
          points: [
            [0, 0],
            [0.5, 0.7],
            [1, 1]
          ]
        }
      }
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

  it('keeps lens corrections on auto unless stated', () => {
    const edits = neutralEdits();
    edits.lens.ca_enabled = true;
    edits.lens.ca_red_scale_x10000 = 30;
    const manifest = editsToManifest(edits);
    expect((manifest.ops.lens_profile as Record<string, unknown>).profile_enabled).toBeNull();
    expect(manifestToEdits(manifest).lens.profile_enabled).toBeNull();
  });

  it('migrates pre-v4 manifests to explicitly disabled lens corrections', () => {
    const edits = manifestToEdits({ schema_version: 3, ops: { exposure: { ev: 1 } } });
    expect(edits.lens.profile_enabled).toBe(false);
  });

  it('preserves luma and color range masks', () => {
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
    expect(roundTrip(edits).masks).toEqual(edits.masks);
  });

  it('round trips polygon components', () => {
    const edits = neutralEdits();
    edits.masks = [
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
    expect(roundTrip(edits).masks).toEqual(edits.masks);
  });

  it('preserves generated mask provenance', () => {
    const edits = neutralEdits();
    edits.masks = [
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
    expect(roundTrip(edits).masks).toEqual(edits.masks);
  });

  it('preserves click points on a refinable mask', () => {
    const edits = neutralEdits();
    edits.masks = [
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
    expect(roundTrip(edits).masks).toEqual(edits.masks);
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
