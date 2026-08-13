import { describe, expect, it } from 'vitest';
import type { GeometryEdits } from '$lib/types/edits';
import type { PreviewMeta } from '$lib/types/preview';
import { neutralPerspective, type PerspectiveEdits } from './perspective';
import {
  displayUvToMaskUv,
  geometryBbox,
  geometryIsIdentity,
  geometryOrientedSize,
  geometryTransformFrom,
  maskUvToDisplayUv,
  type RotateQuarter
} from './geomTransform';

const meta: PreviewMeta = {
  asset_id: 'a',
  width: 600,
  height: 400,
  source_w: 1000,
  source_h: 800,
  renderer: 'cpu',
  is_raw: true,
  histogram: { r: [], g: [], b: [], l: [] }
};

function geometry(patch: Partial<GeometryEdits> = {}): GeometryEdits {
  return {
    rotate: 0,
    flip_h: false,
    flip_v: false,
    rotate_angle: 0,
    crop: null,
    aspect: { kind: 'original' },
    perspective: null,
    ...patch
  };
}

const keystone: PerspectiveEdits = { ...neutralPerspective(), vertical: 40, horizontal: -25 };

describe('geometry uv round trip', () => {
  it.each<[string, Partial<GeometryEdits>]>([
    ['rotate 270', { rotate: 270 }],
    ['both flips', { flip_h: true, flip_v: true }],
    ['fine angle', { rotate_angle: 7.5 }],
    ['angle with crop', { rotate_angle: -12, crop: { x: 0.1, y: 0.2, w: 0.6, h: 0.5 } }],
    ['perspective', { perspective: keystone }],
    [
      'everything',
      {
        rotate: 90,
        flip_h: true,
        rotate_angle: 3,
        crop: { x: 0.05, y: 0.05, w: 0.8, h: 0.7 },
        perspective: keystone
      }
    ]
  ])('maps display uv back to itself with %s', (_name, patch) => {
    const t = geometryTransformFrom(geometry(patch), meta);
    const mask = displayUvToMaskUv(t, [0.3, 0.7]);
    const back = maskUvToDisplayUv(t, mask);
    expect(back[0]).toBeCloseTo(0.3, 5);
    expect(back[1]).toBeCloseTo(0.7, 5);
  });
});

describe('geometryIsIdentity', () => {
  it.each<[string, Partial<GeometryEdits>, boolean]>([
    ['neutral', {}, true],
    ['full crop', { crop: { x: 0, y: 0, w: 1, h: 1 } }, true],
    ['neutral perspective', { perspective: neutralPerspective() }, true],
    ['rotate', { rotate: 180 }, false],
    ['angle', { rotate_angle: 0.5 }, false],
    ['crop', { crop: { x: 0, y: 0, w: 0.9, h: 1 } }, false],
    ['perspective', { perspective: keystone }, false]
  ])('reports %s as %s', (_name, patch, expected) => {
    expect(geometryIsIdentity(geometryTransformFrom(geometry(patch), meta))).toBe(expected);
  });

  it('short circuits the identity transform', () => {
    const t = geometryTransformFrom(geometry(), meta);
    expect(displayUvToMaskUv(t, [0.25, 0.6])).toEqual([0.25, 0.6]);
    expect(maskUvToDisplayUv(t, [0.25, 0.6])).toEqual([0.25, 0.6]);
  });
});

describe('geometryOrientedSize', () => {
  it.each<[RotateQuarter, number, number]>([
    [0, 1000, 800],
    [90, 800, 1000],
    [180, 1000, 800],
    [270, 800, 1000]
  ])('swaps the source size for rotate %s', (rotate, w, h) => {
    const t = geometryTransformFrom(geometry({ rotate }), meta);
    expect(geometryOrientedSize(t)).toEqual({ w, h });
  });
});

describe('geometryBbox', () => {
  it('matches the oriented size when unrotated', () => {
    const t = geometryTransformFrom(geometry(), meta);
    expect(geometryBbox(t)).toEqual({ w: 1000, h: 800 });
  });

  it('grows when the frame is rotated by a fine angle', () => {
    const t = geometryTransformFrom(geometry({ rotate_angle: 20 }), meta);
    const bbox = geometryBbox(t);
    expect(bbox.w).toBeGreaterThan(1000);
    expect(bbox.h).toBeGreaterThan(800);
  });
});
