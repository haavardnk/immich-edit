import { describe, it, expect } from 'vitest';
import {
  IDENTITY_MAT3,
  clampPerspective,
  cornerOffsetsFor,
  limitPerspective,
  mat3Apply,
  neutralPerspective,
  perspectiveCssMatrix,
  perspectiveForward,
  perspectiveInverse,
  perspectiveIsIdentity,
  perspectiveIsUsable,
  perspectiveQuad,
  type PerspectiveEdits
} from './perspective';

function withParams(patch: Partial<PerspectiveEdits>): PerspectiveEdits {
  return { ...neutralPerspective(), ...patch };
}

describe('perspectiveIsIdentity', () => {
  it.each([
    ['null', null, true],
    ['neutral', neutralPerspective(), true],
    ['vertical', withParams({ vertical: 20 }), false],
    ['aspect', withParams({ aspect: 120 }), false],
    [
      'zero corners',
      withParams({
        corners: [
          [0, 0],
          [0, 0],
          [0, 0],
          [0, 0]
        ]
      }),
      true
    ],
    [
      'moved corner',
      withParams({
        corners: [
          [0.1, 0],
          [0, 0],
          [0, 0],
          [0, 0]
        ]
      }),
      false
    ]
  ])('%s', (_label, value, expected) => {
    expect(perspectiveIsIdentity(value)).toBe(expected);
  });
});

describe('perspective matrices', () => {
  it('returns identity for neutral params', () => {
    expect(perspectiveForward(neutralPerspective())).toEqual(IDENTITY_MAT3);
    expect(perspectiveInverse(neutralPerspective())).toEqual(IDENTITY_MAT3);
  });

  it.each([
    withParams({ vertical: 60 }),
    withParams({ horizontal: -45, aspect: 30 }),
    withParams({ vertical: 25, horizontal: 15, aspect: -20 }),
    withParams({
      corners: [
        [0.1, 0.05],
        [-0.08, 0.02],
        [0.04, -0.06],
        [-0.03, -0.02]
      ]
    })
  ])('round trips forward and inverse', (p) => {
    const f = perspectiveForward(p);
    const inv = perspectiveInverse(p);
    for (const point of [
      [0.1, 0.2],
      [0.5, 0.5],
      [0.9, 0.8]
    ] as [number, number][]) {
      const back = mat3Apply(inv, mat3Apply(f, point));
      expect(back[0]).toBeCloseTo(point[0], 4);
      expect(back[1]).toBeCloseTo(point[1], 4);
    }
  });

  it('narrows the bottom edge on positive vertical keystone', () => {
    const quad = perspectiveQuad(withParams({ vertical: 60 }));
    const topWidth = quad[1][0] - quad[0][0];
    const bottomWidth = quad[2][0] - quad[3][0];
    expect(bottomWidth).toBeLessThan(topWidth);
  });

  it('stretches about the centre on aspect', () => {
    const centre = mat3Apply(perspectiveForward(withParams({ aspect: 100 })), [0.5, 0.5]);
    expect(centre[0]).toBeCloseTo(0.5, 6);
    expect(centre[1]).toBeCloseTo(0.5, 6);
    const corner = mat3Apply(perspectiveForward(withParams({ aspect: 100 })), [1, 1]);
    expect(corner[0]).toBeCloseTo(1, 6);
    expect(corner[1]).toBeCloseTo(0.5 + 2 / 9, 6);
  });

  it.each([
    withParams({ vertical: 100 }),
    withParams({ vertical: 100, horizontal: 100, aspect: 40 }),
    withParams({
      corners: [
        [-0.25, -0.25],
        [0.25, -0.25],
        [0.25, 0.25],
        [-0.25, 0.25]
      ]
    })
  ])('keeps the warped quad inside the frame', (p) => {
    for (const c of perspectiveQuad(p)) {
      expect(c[0]).toBeGreaterThanOrEqual(-1e-4);
      expect(c[0]).toBeLessThanOrEqual(1 + 1e-4);
      expect(c[1]).toBeGreaterThanOrEqual(-1e-4);
      expect(c[1]).toBeLessThanOrEqual(1 + 1e-4);
    }
  });

  it('falls back to identity for a mirrored quad', () => {
    const p = withParams({
      corners: [
        [0.9, 0.9],
        [-0.9, 0.9],
        [-0.9, -0.9],
        [0.9, -0.9]
      ]
    });
    expect(perspectiveForward(p)).not.toEqual(IDENTITY_MAT3);
    expect(clampPerspective(p).corners?.[0]).toEqual([0.25, 0.25]);
  });
});

describe('clampPerspective', () => {
  it('limits every range', () => {
    const p = clampPerspective({
      vertical: 500,
      horizontal: -500,
      aspect: 900,
      corners: [
        [9, 9],
        [0, 0],
        [0, 0],
        [0, 0]
      ]
    });
    expect(p.vertical).toBe(100);
    expect(p.horizontal).toBe(-100);
    expect(p.aspect).toBe(100);
    expect(p.corners?.[0]).toEqual([0.25, 0.25]);
  });
});

describe('perspectiveCssMatrix', () => {
  it('is the identity transform for a neutral matrix', () => {
    expect(perspectiveCssMatrix(IDENTITY_MAT3, 400, 300)).toBe(
      'matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1)'
    );
  });

  it('maps pixel corners the same way the uv matrix maps uv corners', () => {
    const m = perspectiveForward(withParams({ vertical: 40, aspect: 20 }));
    const css = perspectiveCssMatrix(m, 400, 300);
    const v = css
      .slice('matrix3d('.length, -1)
      .split(',')
      .map((s) => Number(s));
    const px = [v[0] * -200 + v[4] * -150 + v[12], v[1] * -200 + v[5] * -150 + v[13]];
    const w = v[3] * -200 + v[7] * -150 + v[15];
    const uv = mat3Apply(m, [0, 0]);
    expect(px[0] / w).toBeCloseTo((uv[0] - 0.5) * 400, 4);
    expect(px[1] / w).toBeCloseTo((uv[1] - 0.5) * 300, 4);
  });
});

describe('cornerOffsetsFor', () => {
  it('moves the dragged corner onto the requested uv point', () => {
    const p = withParams({ vertical: 20 });
    const target: [number, number] = [0.12, 0.08];
    const next = { ...p, corners: cornerOffsetsFor(p, 0, target) };
    const moved = mat3Apply(perspectiveForward(next), [0, 0]);
    expect(moved[0]).toBeCloseTo(target[0], 5);
    expect(moved[1]).toBeCloseTo(target[1], 5);
  });

  it('clamps the offset to the corner limit', () => {
    const offsets = cornerOffsetsFor(neutralPerspective(), 2, [-5, -5]);
    expect(offsets[2]).toEqual([-0.25, -0.25]);
  });
});

describe('limitPerspective', () => {
  const start = withParams({ vertical: 100, horizontal: -50, aspect: -100 });
  const target = withParams({ vertical: 100, horizontal: -100, aspect: -100 });

  it('stops at the last usable value instead of collapsing to identity', () => {
    expect(perspectiveIsUsable(start)).toBe(true);
    expect(perspectiveIsUsable(target)).toBe(false);

    const limited = limitPerspective(start, target);
    expect(perspectiveIsUsable(limited)).toBe(true);
    expect(limited.horizontal).toBeLessThan(-50);
    expect(limited.horizontal).toBeGreaterThan(-100);
    expect(perspectiveForward(limited)).not.toEqual(IDENTITY_MAT3);
  });

  it('passes usable targets through untouched', () => {
    const target = withParams({ vertical: 30 });
    expect(limitPerspective(neutralPerspective(), target)).toEqual(target);
  });
});
