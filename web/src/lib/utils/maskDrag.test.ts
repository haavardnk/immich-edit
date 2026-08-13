import { describe, expect, it } from 'vitest';
import type { MaskComponentKind } from '$lib/types/edits';
import { draggedKind } from './maskDrag';

const linear: Extract<MaskComponentKind, { kind: 'linear' }> = {
  kind: 'linear',
  p0: { x: 0, y: 0 },
  p1: { x: 1, y: 0 },
  feather: 0.5
};

const radial: Extract<MaskComponentKind, { kind: 'radial' }> = {
  kind: 'radial',
  center: { x: 0.5, y: 0.5 },
  radius_xy: { x: 0.4, y: 0.4 },
  feather: 0.5
};

const polygon: Extract<MaskComponentKind, { kind: 'polygon' }> = {
  kind: 'polygon',
  feather: 0,
  points: [
    { x: 0, y: 0 },
    { x: 1, y: 0 },
    { x: 1, y: 1 }
  ]
};

describe('draggedKind', () => {
  it('moves a linear gradient by the pointer delta and clamps to the frame', () => {
    const next = draggedKind(
      linear,
      {
        kind: 'linear-move',
        startP0: { x: 0.1, y: 0.1 },
        startP1: { x: 0.4, y: 0.4 },
        downAtN: { x: 0.5, y: 0.5 }
      },
      { x: 1.4, y: 0.6 }
    );
    expect(next?.kind).toBe('linear');
    const moved = next as Extract<MaskComponentKind, { kind: 'linear' }>;
    expect(moved.p0.x).toBeCloseTo(1, 6);
    expect(moved.p0.y).toBeCloseTo(0.2, 6);
    expect(moved.p1).toEqual({ x: 1, y: 0.5 });
  });

  it('derives linear feather from the distance along the gradient axis', () => {
    const next = draggedKind(linear, { kind: 'linear-feather' }, { x: 0.75, y: 0 });
    expect(next).toEqual({ ...linear, feather: 0.5 });
  });

  it('keeps a radial radius above the minimum', () => {
    const next = draggedKind(radial, { kind: 'radial-rx', sign: 1 }, { x: 0.5, y: 0.5 });
    expect(next).toEqual({ ...radial, radius_xy: { x: 0.005, y: 0.4 } });
  });

  it('derives radial feather from the normalized distance to the edge', () => {
    const next = draggedKind(radial, { kind: 'radial-feather' }, { x: 0.7, y: 0.5 });
    expect(next?.kind).toBe('radial');
    expect((next as Extract<MaskComponentKind, { kind: 'radial' }>).feather).toBeCloseTo(0.5, 6);
  });

  it('moves one polygon vertex and leaves the rest alone', () => {
    const next = draggedKind(polygon, { kind: 'polygon-vertex', index: 1 }, { x: 0.5, y: 0.25 });
    expect(next).toEqual({
      kind: 'polygon',
      feather: 0,
      points: [
        { x: 0, y: 0 },
        { x: 0.5, y: 0.25 },
        { x: 1, y: 1 }
      ]
    });
  });

  it('ignores a drag meant for another shape kind', () => {
    expect(draggedKind(linear, { kind: 'radial-center' }, { x: 0.5, y: 0.5 })).toBeNull();
  });
});
