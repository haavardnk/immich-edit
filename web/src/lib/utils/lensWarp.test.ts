import { describe, it, expect } from 'vitest';
import {
  lensWarpActive,
  lensWarpFromEdits,
  maskUvToSceneUv,
  sceneUvToMaskUv,
  type LensWarpParams
} from './lensWarp';

const NEUTRAL: LensWarpParams = { k1: 0, k2: 0, k3: 0, zoom: 1, width: 4000, height: 3000 };

describe('lensWarpActive', () => {
  it('is false for the neutral params', () => {
    expect(lensWarpActive(NEUTRAL)).toBe(false);
  });

  it('is true when any coefficient is non-zero', () => {
    expect(lensWarpActive({ ...NEUTRAL, k1: 0.1 })).toBe(true);
    expect(lensWarpActive({ ...NEUTRAL, zoom: 1.2 })).toBe(true);
  });
});

describe('lensWarpFromEdits', () => {
  const lens = {
    profile_enabled: true,
    constrain_crop: false,
    distortion_amount: 50,
    k1: 0.2,
    k2: -0.1,
    k3: 0.05
  };

  it('returns neutral params when the profile is disabled', () => {
    const p = lensWarpFromEdits({ ...lens, profile_enabled: false }, 4000, 3000);
    expect(p).toEqual({ k1: 0, k2: 0, k3: 0, zoom: 1, width: 4000, height: 3000 });
  });

  it('scales coefficients by distortion_amount / 100', () => {
    const p = lensWarpFromEdits(lens, 4000, 3000);
    expect(p.k1).toBeCloseTo(0.1, 10);
    expect(p.k2).toBeCloseTo(-0.05, 10);
    expect(p.k3).toBeCloseTo(0.025, 10);
    expect(p.zoom).toBe(1);
  });

  it('solves a zoom when constrain_crop is set', () => {
    const p = lensWarpFromEdits({ ...lens, constrain_crop: true }, 4000, 3000);
    expect(p.zoom).toBeGreaterThan(0);
    expect(p.zoom).toBeLessThanOrEqual(1);
  });
});

describe('mask/scene UV round-trip', () => {
  it('is identity for neutral params', () => {
    expect(maskUvToSceneUv(NEUTRAL, [0.3, 0.7])).toEqual([0.3, 0.7]);
    expect(sceneUvToMaskUv(NEUTRAL, [0.3, 0.7])).toEqual([0.3, 0.7]);
  });

  it('inverts maskUvToSceneUv via sceneUvToMaskUv', () => {
    const p: LensWarpParams = {
      k1: 0.15,
      k2: -0.05,
      k3: 0.02,
      zoom: 0.9,
      width: 4000,
      height: 3000
    };
    const mask: [number, number] = [0.62, 0.4];
    const scene = maskUvToSceneUv(p, mask);
    const back = sceneUvToMaskUv(p, scene);
    expect(back[0]).toBeCloseTo(mask[0], 3);
    expect(back[1]).toBeCloseTo(mask[1], 3);
  });

  it('keeps the image center fixed', () => {
    const p: LensWarpParams = { k1: 0.2, k2: 0, k3: 0, zoom: 0.95, width: 4000, height: 3000 };
    expect(maskUvToSceneUv(p, [0.5, 0.5])).toEqual([0.5, 0.5]);
    expect(sceneUvToMaskUv(p, [0.5, 0.5])).toEqual([0.5, 0.5]);
  });
});
