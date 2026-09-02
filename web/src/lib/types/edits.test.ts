import { describe, it, expect } from 'vitest';
import {
  neutralEdits,
  isIdentity,
  isNonGeometryIdentity,
  curvesEditsIsIdentity,
  neutralSharpenAmount,
  effectiveLens,
  originalPreviewEdits,
  resetDevelopEdits
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
