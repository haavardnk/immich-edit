import { describe, it, expect } from 'vitest';
import {
  neutralEdits,
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

  it('treats mask-only edits as non-identity', () => {
    const e = neutralEdits();
    e.masks = [
      {
        id: 'layer',
        name: 'Range',
        enabled: true,
        color: '#ff3b30',
        amount: 1,
        components: [
          {
            id: 'luma',
            enabled: true,
            mode: 'add',
            opacity: 1,
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
        components: [
          {
            id: 'luma',
            enabled: true,
            mode: 'add',
            opacity: 1,
            invert: false,
            kind: { kind: 'luma_range', min: 0.2, max: 0.8, softness: 0.1 },
            source: 'manual'
          },
          {
            id: 'color',
            enabled: true,
            mode: 'intersect',
            opacity: 0.9,
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
