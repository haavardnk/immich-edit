import { describe, expect, it } from 'vitest';
import { centredPoint, clampedPoint, elementPoint, normalizedPoint } from './point.ts';

const BOX = { left: 10, top: 20, width: 200, height: 100 };

const element = { getBoundingClientRect: () => BOX } as unknown as Element;

describe('point mapping', () => {
  it('maps client coordinates to element pixels', () => {
    expect(elementPoint({ clientX: 60, clientY: 45 }, BOX)).toEqual({ x: 50, y: 25 });
  });

  it('reads the box from an element', () => {
    expect(elementPoint({ clientX: 10, clientY: 20 }, element)).toEqual({ x: 0, y: 0 });
    expect(normalizedPoint({ clientX: 110, clientY: 70 }, element)).toEqual({ x: 0.5, y: 0.5 });
  });

  it('normalises without clamping', () => {
    expect(normalizedPoint({ clientX: 410, clientY: 0 }, BOX)).toEqual({ x: 2, y: -0.2 });
  });

  it('guards an empty box against division by zero', () => {
    const empty = { left: 0, top: 0, width: 0, height: 0 };
    expect(normalizedPoint({ clientX: 3, clientY: 4 }, empty)).toEqual({ x: 3, y: 4 });
  });

  it('clamps into the unit square', () => {
    expect(clampedPoint({ clientX: 410, clientY: 0 }, BOX)).toEqual({ x: 1, y: 0 });
    expect(clampedPoint({ clientX: 60, clientY: 45 }, BOX)).toEqual({ x: 0.25, y: 0.25 });
  });

  it('measures the offset from the box centre', () => {
    expect(centredPoint({ clientX: 110, clientY: 70 }, BOX)).toEqual({ x: 0, y: 0 });
    expect(centredPoint({ clientX: 10, clientY: 90 }, element)).toEqual({ x: -100, y: 20 });
  });
});
