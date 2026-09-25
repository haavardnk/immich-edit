import { describe, expect, it } from 'vitest';
import type { MaskComponentKind } from '$lib/types/edits';
import { draggedKind, nudgedKind, radialAxes } from './maskDrag';

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

  it('pins radial feather at full when the handle reaches the centre', () => {
    const next = draggedKind(radial, { kind: 'radial-feather' }, radial.center);
    expect((next as Extract<MaskComponentKind, { kind: 'radial' }>).feather).toBe(1);
  });

  it('moves one polygon vertex and leaves the rest alone', () => {
    const next = draggedKind(
      polygon,
      { kind: 'polygon-vertex', index: 1, origin: { x: 1, y: 0 } },
      { x: 0.5, y: 0.25 }
    );
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

describe('nudgedKind', () => {
  const toPx = (v: { x: number; y: number }) => ({ x: v.x * 1000, y: v.y * 500 });
  const fromPx = (x: number, y: number) => ({ x: x / 1000, y: y / 500 });

  it.each<[string, MaskComponentKind, MaskComponentKind]>([
    ['a radial centre', radial, { ...radial, center: { x: 0.51, y: 0.49 } }],
    [
      'both linear ends',
      { ...linear, p1: { x: 0.5, y: 0.5 } },
      { ...linear, p0: { x: 0.01, y: 0 }, p1: { x: 0.51, y: 0.49 } }
    ],
    [
      'every polygon corner',
      {
        ...polygon,
        points: [
          { x: 0.2, y: 0.2 },
          { x: 0.4, y: 0.4 }
        ]
      },
      {
        ...polygon,
        points: [
          { x: 0.21, y: 0.19 },
          { x: 0.41, y: 0.39 }
        ]
      }
    ]
  ])('moves %s by display pixels', (_name, kind, expected) => {
    const next = nudgedKind(kind, 10, -5, toPx, fromPx);
    expect(JSON.stringify(next, (_k, v) => (typeof v === 'number' ? +v.toFixed(6) : v))).toBe(
      JSON.stringify(expected)
    );
  });

  it('leaves shapes without a position alone', () => {
    expect(nudgedKind({ kind: 'brush', raster_id: 'r' }, 1, 0, toPx, fromPx)).toBeNull();
  });
});

describe('rotated radials', () => {
  const ellipse: Extract<MaskComponentKind, { kind: 'radial' }> = {
    kind: 'radial',
    center: { x: 0.5, y: 0.5 },
    radius_xy: { x: 0.2, y: 0.1 },
    feather: 0.2
  };

  it.each([
    [{ x: 0.5, y: 0.8 }, 1, 90],
    [{ x: 0.2, y: 0.5 }, 1, -180],
    [{ x: 0.7, y: 0.7 }, 2, (Math.atan2(0.2, 0.4) * 180) / Math.PI]
  ])('the rotation grip points the x axis at %o on aspect %s', (at, aspect, want) => {
    const next = draggedKind(ellipse, { kind: 'radial-rotate' }, at, { aspect, shift: false });
    if (next?.kind !== 'radial') throw new Error('not radial');
    expect(next.angle).toBeCloseTo(want, 6);
  });

  it('leaves the angle out when the grip lines up with the frame again', () => {
    const tilted = { ...ellipse, angle: 30 };
    const next = draggedKind(
      tilted,
      { kind: 'radial-rotate' },
      { x: 0.9, y: 0.5 },
      { aspect: 1.5, shift: false }
    );
    expect(next).toEqual(ellipse);
  });

  it('measures the x radius along the rotated axis in pixels', () => {
    const tilted = { ...ellipse, angle: 90 };
    const next = draggedKind(
      tilted,
      { kind: 'radial-rx', sign: 1 },
      { x: 0.5, y: 0.8 },
      { aspect: 2, shift: false }
    );
    if (next?.kind !== 'radial') throw new Error('not radial');
    expect(next.radius_xy.x).toBeCloseTo(0.15, 6);
    expect(next.radius_xy.y).toBe(0.1);
  });

  it('places the rotated axes in uv so they are perpendicular in pixels', () => {
    const axes = radialAxes({ ...ellipse, angle: 30 }, 1.5);
    expect(axes.x.x * 1.5 * axes.y.x * 1.5 + axes.x.y * axes.y.y).toBeCloseTo(0, 9);
    expect(Math.hypot(axes.x.x * 1.5, axes.x.y)).toBeCloseTo(0.2 * 1.5, 9);
  });
});

describe('Shift while dragging a mask handle', () => {
  const shift = (aspect: number) => ({ aspect, shift: true });

  it.each([
    [{ kind: 'radial-rx', sign: 1 } as const, { x: 0.8, y: 0.5 }, { x: 0.3, y: 0.45 }],
    [{ kind: 'radial-ry', sign: 1 } as const, { x: 0.5, y: 0.8 }, { x: 0.2, y: 0.3 }]
  ])('%o keeps the radial round in pixels', (drag, at, radius) => {
    const next = draggedKind(radial, drag, at, shift(1.5));
    if (next?.kind !== 'radial') throw new Error('not radial');
    expect(next.radius_xy.x).toBeCloseTo(radius.x, 9);
    expect(next.radius_xy.y).toBeCloseTo(radius.y, 9);
  });

  it.each([
    [
      { x: 1, y: 0.1 },
      { x: Math.hypot(1, 0.1), y: 0 }
    ],
    [
      { x: 0.3, y: 0.28 },
      { x: Math.hypot(0.3, 0.28) / Math.SQRT2, y: Math.hypot(0.3, 0.28) / Math.SQRT2 }
    ]
  ])('snaps a linear end dragged to %o onto the nearest 45° step', (at, want) => {
    const next = draggedKind(linear, { kind: 'linear-p1' }, at, shift(1));
    if (next?.kind !== 'linear') throw new Error('not linear');
    expect(next.p1.x).toBeCloseTo(want.x, 9);
    expect(next.p1.y).toBeCloseTo(want.y, 9);
    expect(next.p0).toEqual(linear.p0);
  });

  it.each([
    [
      { x: 0.7, y: 0.05 },
      { x: 0.7, y: 0 }
    ],
    [
      { x: 0.95, y: 0.3 },
      { x: 1, y: 0.3 }
    ]
  ])('locks a polygon corner dragged to %o to its larger axis', (at, want) => {
    const next = draggedKind(
      polygon,
      { kind: 'polygon-vertex', index: 1, origin: { x: 1, y: 0 } },
      at,
      shift(1)
    );
    if (next?.kind !== 'polygon') throw new Error('not polygon');
    expect(next.points[1]).toEqual(want);
  });
});
