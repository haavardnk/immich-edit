import { describe, it, expect } from 'vitest';
import {
  degToRad,
  rotatedBbox,
  aspectRatioFor,
  pointInWarpedSource,
  cropRectInsideWarpedSource,
  largestInscribedRect,
  refitCropAtAspect,
  constrainCropRect,
  croppedOutputSize
} from './geom';
import type { CropRect } from '../types/edits';
import { FULL_CROP, neutralEdits } from '../types/edits';

describe('degToRad', () => {
  it('converts degrees to radians', () => {
    expect(degToRad(180)).toBeCloseTo(Math.PI, 10);
    expect(degToRad(0)).toBe(0);
    expect(degToRad(90)).toBeCloseTo(Math.PI / 2, 10);
  });
});

describe('rotatedBbox', () => {
  it('returns the source size at angle 0', () => {
    const b = rotatedBbox(100, 60, 0);
    expect(b.w).toBeCloseTo(100, 6);
    expect(b.h).toBeCloseTo(60, 6);
  });

  it('swaps dimensions at 90 degrees', () => {
    const b = rotatedBbox(100, 60, 90);
    expect(b.w).toBeCloseTo(60, 6);
    expect(b.h).toBeCloseTo(100, 6);
  });

  it('grows for an intermediate angle', () => {
    const b = rotatedBbox(100, 100, 45);
    expect(b.w).toBeCloseTo(Math.sqrt(2) * 100, 4);
    expect(b.h).toBeCloseTo(Math.sqrt(2) * 100, 4);
  });
});

describe('aspectRatioFor', () => {
  it('returns null for free aspect', () => {
    expect(aspectRatioFor({ kind: 'free' }, 100, 50)).toBeNull();
  });

  it('returns source ratio for original', () => {
    expect(aspectRatioFor({ kind: 'original' }, 100, 50)).toBe(2);
  });

  it('returns num/den for ratio', () => {
    expect(aspectRatioFor({ kind: 'ratio', num: 16, den: 9 }, 100, 50)).toBeCloseTo(16 / 9, 10);
  });

  it('returns null for a degenerate ratio', () => {
    expect(aspectRatioFor({ kind: 'ratio', num: 0, den: 9 }, 100, 50)).toBeNull();
    expect(aspectRatioFor({ kind: 'ratio', num: 16, den: 0 }, 100, 50)).toBeNull();
  });
});

describe('pointInWarpedSource', () => {
  it('accepts the center point', () => {
    const b = rotatedBbox(100, 60, 30);
    expect(pointInWarpedSource({ x: b.w / 2, y: b.h / 2 }, 100, 60, 30)).toBe(true);
  });

  it('rejects a far corner of the bbox at an angle', () => {
    const b = rotatedBbox(100, 60, 30);
    expect(pointInWarpedSource({ x: 0, y: 0 }, 100, 60, 30)).toBe(false);
    expect(b.w).toBeGreaterThan(100);
  });
});

describe('cropRectInsideWarpedSource', () => {
  it('accepts the full crop at angle 0', () => {
    expect(cropRectInsideWarpedSource(FULL_CROP, 100, 60, 0)).toBe(true);
  });

  it('rejects the full crop once rotated', () => {
    expect(cropRectInsideWarpedSource(FULL_CROP, 100, 60, 20)).toBe(false);
  });
});

describe('largestInscribedRect', () => {
  it('fits inside the rotated source', () => {
    const rect = largestInscribedRect(100, 60, 15, 100 / 60);
    expect(cropRectInsideWarpedSource(rect, 100, 60, 15)).toBe(true);
  });

  it('matches the requested aspect ratio', () => {
    const aspect = 16 / 9;
    const rect = largestInscribedRect(100, 60, 10, aspect);
    const bbox = rotatedBbox(100, 60, 10);
    const ratio = (rect.w * bbox.w) / (rect.h * bbox.h);
    expect(ratio).toBeCloseTo(aspect, 2);
  });

  it('returns the full frame at angle 0 with source aspect', () => {
    const rect = largestInscribedRect(100, 60, 0, 100 / 60);
    expect(rect.w).toBeCloseTo(1, 3);
    expect(rect.h).toBeCloseTo(1, 3);
  });
});

describe('refitCropAtAspect', () => {
  it('produces a crop inside the rotated source', () => {
    const prev: CropRect = { x: 0.3, y: 0.3, w: 0.4, h: 0.4 };
    const rect = refitCropAtAspect(prev, 100, 60, 12, 1);
    expect(cropRectInsideWarpedSource(rect, 100, 60, 12)).toBe(true);
  });
});

describe('constrainCropRect', () => {
  it('passes through a valid crop unchanged', () => {
    const candidate: CropRect = { x: 0.2, y: 0.2, w: 0.5, h: 0.5 };
    const out = constrainCropRect(candidate, null, 100, 60, 0);
    expect(out).toEqual(candidate);
  });

  it('clamps a crop that exceeds the unit square', () => {
    const candidate: CropRect = { x: 0.8, y: 0.8, w: 0.5, h: 0.5 };
    const out = constrainCropRect(candidate, null, 100, 60, 0);
    expect(out.x + out.w).toBeLessThanOrEqual(1 + 1e-9);
    expect(out.y + out.h).toBeLessThanOrEqual(1 + 1e-9);
  });

  it('returns a valid crop when rotation invalidates the candidate', () => {
    const candidate: CropRect = { x: 0, y: 0, w: 1, h: 1 };
    const out = constrainCropRect(candidate, null, 100, 60, 20);
    expect(cropRectInsideWarpedSource(out, 100, 60, 20)).toBe(true);
  });
});

describe('croppedOutputSize', () => {
  it.each<[string, Partial<ReturnType<typeof neutralEdits>['geometry']>, number, number]>([
    ['keeps the source without geometry', {}, 6000, 4000],
    ['halves a centred crop', { crop: { x: 0.25, y: 0.25, w: 0.5, h: 0.5 } }, 3000, 2000],
    ['swaps sides on a quarter turn', { rotate: 90 }, 4000, 6000],
    ['grows the frame for a straighten angle', { rotate_angle: 90 }, 4000, 6000],
    ['crops the rotated frame', { rotate: 270, crop: { x: 0, y: 0, w: 0.5, h: 1 } }, 2000, 6000]
  ])('%s', (_name, patch, w, h) => {
    const geometry = { ...neutralEdits().geometry, ...patch };
    expect(croppedOutputSize(geometry, 6000, 4000)).toEqual({ w, h });
  });
});
