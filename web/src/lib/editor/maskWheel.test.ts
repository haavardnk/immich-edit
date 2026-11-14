import type { MaskComponentKind } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import {
  featheredKind,
  resizedKind,
  shapeExtentLabel,
  shapeFeatherLabel,
  wheeledKind
} from './maskWheel';

const radial: MaskComponentKind = {
  kind: 'radial',
  center: { x: 0.5, y: 0.5 },
  radius_xy: { x: 0.2, y: 0.3 },
  feather: 0.5
};

const linear: MaskComponentKind = {
  kind: 'linear',
  p0: { x: 0.4, y: 0.5 },
  p1: { x: 0.6, y: 0.5 },
  feather: 0.2
};

const polygon: MaskComponentKind = {
  kind: 'polygon',
  points: [
    { x: 0.2, y: 0.2 },
    { x: 0.4, y: 0.2 },
    { x: 0.4, y: 0.6 }
  ],
  feather: 0.05
};

describe('resizedKind', () => {
  it('scales both radial radii by one notch', () => {
    const next = resizedKind(radial, 1, 1.5);
    expect(next).toMatchObject({ radius_xy: { x: 0.2 * 1.1, y: 0.3 * 1.1 } });
    expect(next).toMatchObject({ center: radial.center, feather: 0.5 });
  });

  it('shrinks a linear gradient about its midpoint', () => {
    const next = resizedKind(linear, -1, 1);
    if (next?.kind !== 'linear') throw new Error('expected linear');
    expect(next.p0.x).toBeCloseTo(0.5 - 0.1 / 1.1);
    expect(next.p1.x).toBeCloseTo(0.5 + 0.1 / 1.1);
    expect(next.p0.y).toBe(0.5);
  });

  it('scales polygon corners about the centre of their bounds', () => {
    const next = resizedKind(polygon, 2, 1);
    if (next?.kind !== 'polygon') throw new Error('expected polygon');
    const f = 1.1 ** 2;
    expect(next.points[0]?.x).toBeCloseTo(0.3 - 0.1 * f);
    expect(next.points[2]?.y).toBeCloseTo(0.4 + 0.2 * f);
  });

  it('stops at the smallest extent', () => {
    const next = resizedKind(radial, -200, 1);
    if (next?.kind !== 'radial') throw new Error('expected radial');
    expect(Math.max(next.radius_xy.x, next.radius_xy.y)).toBeCloseTo(0.005);
  });

  it('ignores kinds without a size', () => {
    expect(resizedKind({ kind: 'brush', raster_id: 'r' }, 1, 1)).toBeNull();
    expect(resizedKind({ kind: 'luma_range', min: 0, max: 1, softness: 0 }, 1, 1)).toBeNull();
  });
});

describe('featheredKind', () => {
  it('steps gradients by a twentieth of the range', () => {
    expect(featheredKind(radial, 1)).toMatchObject({ feather: 0.55 });
    expect(featheredKind(linear, -10)).toMatchObject({ feather: 0 });
  });

  it('uses the polygon feather range', () => {
    const next = featheredKind(polygon, 1);
    expect(next && 'feather' in next ? next.feather : 0).toBeCloseTo(0.075);
    expect(featheredKind(polygon, 100)).toMatchObject({ feather: 0.5 });
  });

  it('picks the axis', () => {
    expect(wheeledKind(radial, 'feather', 1, 1)).toMatchObject({ feather: 0.55 });
    expect(wheeledKind(radial, 'size', 1, 1)).toMatchObject({ feather: 0.5 });
  });
});

describe('labels', () => {
  it('names the extent in source pixels on the short edge', () => {
    expect(shapeExtentLabel(radial, { w: 3000, h: 2000 })).toBe('600 px');
    expect(shapeExtentLabel(linear, { w: 3000, h: 2000 })).toBe('600 px');
  });

  it('formats feather with the range format', () => {
    expect(shapeFeatherLabel(radial)).toBe('0.50');
    expect(shapeFeatherLabel(polygon)).toBe('0.050');
  });
});
