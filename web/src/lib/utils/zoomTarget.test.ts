import { describe, expect, it } from 'vitest';
import type { FaceBox } from '$lib/api/faces';
import type { Edits, GeometryEdits } from '$lib/types/edits';
import { neutralEdits } from '$lib/types/edits';
import { viewTransform } from './canvasCoords';
import { faceTargets, nextTargetIndex, panForTarget, sharpestCell } from './zoomTarget';

const dims = { source_w: 1000, source_h: 800 };

function face(x: number, y: number, w: number, h: number): FaceBox {
  return { source_w: dims.source_w, source_h: dims.source_h, x, y, w, h };
}

function editsWith(geometry: Partial<GeometryEdits>): Edits {
  const base = neutralEdits();
  return { ...base, geometry: { ...base.geometry, ...geometry } };
}

describe('faceTargets', () => {
  it('orders faces from largest to smallest', () => {
    const view = viewTransform(neutralEdits(), dims);
    const targets = faceTargets([face(0.1, 0.1, 0.05, 0.05), face(0.6, 0.6, 0.2, 0.2)], view);
    expect(targets).toEqual([
      { u: 0.7, v: 0.7 },
      { u: 0.125, v: 0.125 }
    ]);
  });

  it('maps face centres through the saved crop', () => {
    const view = viewTransform(editsWith({ crop: { x: 0.5, y: 0.5, w: 0.5, h: 0.5 } }), dims);
    const targets = faceTargets([face(0.7, 0.7, 0.1, 0.1)], view);
    expect(targets).toHaveLength(1);
    expect(targets[0]?.u).toBeCloseTo(0.5, 6);
    expect(targets[0]?.v).toBeCloseTo(0.5, 6);
  });

  it('drops faces cropped out of the rendered image', () => {
    const view = viewTransform(editsWith({ crop: { x: 0, y: 0, w: 0.25, h: 0.25 } }), dims);
    expect(faceTargets([face(0.7, 0.7, 0.1, 0.1)], view)).toEqual([]);
  });

  it('maps face centres through a quarter rotation', () => {
    const view = viewTransform(editsWith({ rotate: 90 }), dims);
    const targets = faceTargets([face(0.1, 0.2, 0.2, 0.2)], view);
    expect(targets[0]?.u).toBeCloseTo(0.7, 6);
    expect(targets[0]?.v).toBeCloseTo(0.2, 6);
  });
});

describe('nextTargetIndex', () => {
  it.each<[number | null, number, number | null]>([
    [null, 0, null],
    [null, 2, 0],
    [0, 2, 1],
    [1, 2, null]
  ])('steps %s of %i targets', (index, count, expected) => {
    expect(nextTargetIndex(index, count)).toBe(expected);
  });
});

describe('panForTarget', () => {
  const frame = { width: 400, height: 300 };

  it('centres the target in the viewport', () => {
    expect(panForTarget({ u: 0.25, v: 0.75 }, frame, 400, 300, 2)).toEqual({
      panX: 200,
      panY: -150
    });
  });

  it('clamps so the zoomed frame never leaves the viewport', () => {
    expect(panForTarget({ u: 0, v: 1 }, frame, 400, 300, 2)).toEqual({ panX: 200, panY: -150 });
  });

  it('holds the centre when the frame still fits', () => {
    const pan = panForTarget({ u: 0.1, v: 0.9 }, frame, 400, 300, 1);
    expect(pan.panX).toBeCloseTo(0);
    expect(pan.panY).toBeCloseTo(0);
  });
});

describe('sharpestCell', () => {
  it('finds the cell holding the detail', () => {
    const w = 64;
    const h = 64;
    const gray = new Float32Array(w * h);
    for (let y = 48; y < 56; y++) {
      for (let x = 8; x < 16; x++) {
        gray[y * w + x] = (x + y) % 2 === 0 ? 255 : 0;
      }
    }
    expect(sharpestCell(gray, w, h)).toEqual({ u: 1.5 / 8, v: 6.5 / 8 });
  });

  it('returns null for a flat image', () => {
    expect(sharpestCell(new Float32Array(64 * 64).fill(128), 64, 64)).toBeNull();
  });
});
