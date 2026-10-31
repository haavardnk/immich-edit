import { sourceToStage, stageFrame } from '#lib/editor/geometryStage.ts';
import { describe, expect, it } from 'vitest';
import { CORNER_LIMIT, perspectiveIsUsable, type Point } from './perspective';
import {
  classifyGuide,
  solveUpright,
  withGuide,
  type GuideKind,
  type UprightGuide,
  type UprightSolution
} from './upright';

const W = 600;
const H = 400;

function guide(kind: GuideKind, from: Point, to: Point): UprightGuide {
  return { kind, from, to };
}

const LEFT = guide('vertical', [0.3, 0.9], [0.35, 0.1]);
const RIGHT = guide('vertical', [0.7, 0.9], [0.65, 0.1]);
const LEAN_LEFT = guide('vertical', [0.2, 0.9], [0.28, 0.1]);
const LEAN_RIGHT = guide('vertical', [0.6, 0.9], [0.64, 0.1]);
const TOP = guide('horizontal', [0.1, 0.3], [0.9, 0.35]);
const BOTTOM = guide('horizontal', [0.1, 0.7], [0.9, 0.62]);
const TILTED_V = guide('vertical', [0.4, 0.9], [0.45, 0.1]);
const TILTED_H = guide('horizontal', [0.1, 0.5], [0.9, 0.55]);

function solved(guides: UprightGuide[], aspect = 0): UprightSolution {
  const solution = solveUpright(guides, W, H, aspect);
  if (!solution) throw new Error('expected a solution');
  return solution;
}

function axisError(solution: UprightSolution, g: UprightGuide): number {
  const frame = stageFrame({
    bboxW: W,
    bboxH: H,
    orientedW: W,
    orientedH: H,
    angle: solution.angle,
    perspective: solution.perspective
  });
  const a = sourceToStage(frame, g.from);
  const b = sourceToStage(frame, g.to);
  const dx = b[0] - a[0];
  const dy = b[1] - a[1];
  const off = g.kind === 'vertical' ? dx : dy;
  return Math.abs(off) / Math.hypot(dx, dy);
}

describe('classifyGuide', () => {
  it.each([
    ['vertical', [100, 10], [120, 300], 'vertical'],
    ['horizontal', [10, 100], [300, 140], 'horizontal'],
    ['diagonal', [10, 10], [200, 200], null],
    ['short', [10, 10], [10, 15], null]
  ] as const)('classifies a %s line', (_, from, to, kind) => {
    expect(classifyGuide([...from], [...to], Math.hypot(W, H))).toBe(kind);
  });
});

describe('withGuide', () => {
  it('replaces the oldest guide of a full family and keeps the other family', () => {
    const guides = withGuide(withGuide(withGuide([], LEFT), TOP), RIGHT);
    expect(withGuide(guides, TILTED_V)).toEqual([TOP, RIGHT, TILTED_V]);
  });
});

describe('solveUpright', () => {
  it('needs at least two guides', () => {
    expect(solveUpright([LEFT], W, H, 0)).toBeNull();
  });

  it('leaves parallel axis-aligned guides untouched', () => {
    const solution = solved([
      guide('vertical', [0.2, 0.9], [0.2, 0.1]),
      guide('vertical', [0.8, 0.9], [0.8, 0.1])
    ]);
    expect(solution.angle).toBeCloseTo(0);
    expect(solution.perspective.vertical).toBeCloseTo(0);
    expect(solution.perspective.horizontal).toBeCloseTo(0);
    expect(solution.perspective.corners).toBeNull();
  });

  it('corrects symmetric converging verticals with vertical keystone only', () => {
    const solution = solved([LEFT, RIGHT]);
    expect(solution.perspective.vertical).not.toBeCloseTo(0);
    expect(solution.perspective.horizontal).toBeCloseTo(0);
    expect(solution.angle).toBeCloseTo(0);
    expect(axisError(solution, LEFT)).toBeLessThan(1e-6);
    expect(axisError(solution, RIGHT)).toBeLessThan(1e-6);
  });

  it.each([
    ['two leaning verticals', [LEAN_LEFT, LEAN_RIGHT]],
    ['two horizontals', [TOP, BOTTOM]],
    ['two of each', [LEFT, RIGHT, TOP, BOTTOM]],
    ['two verticals and one horizontal', [LEAN_LEFT, LEAN_RIGHT, TOP]],
    ['two horizontals and one vertical', [TOP, BOTTOM, LEAN_LEFT]],
    ['one of each', [TILTED_V, TILTED_H]]
  ])('makes every guide axis-aligned for %s', (_, guides) => {
    const solution = solved(guides);
    expect(perspectiveIsUsable(solution.perspective)).toBe(true);
    for (const g of guides) expect(axisError(solution, g)).toBeLessThan(1e-6);
  });

  it('keeps the aspect stretch', () => {
    const solution = solved([LEAN_LEFT, LEAN_RIGHT, TOP], 20);
    expect(solution.perspective.aspect).toBe(20);
    for (const g of [LEAN_LEFT, LEAN_RIGHT, TOP]) {
      expect(axisError(solution, g)).toBeLessThan(1e-6);
    }
  });

  it('scales a shear past the corner limit back while keeping the primary family exact', () => {
    const steep = guide('horizontal', [0.1, 0.1], [0.9, 0.91]);
    const solution = solved([LEFT, RIGHT, steep]);
    const corners = solution.perspective.corners;
    expect(corners).not.toBeNull();
    for (const [x, y] of corners ?? []) {
      expect(Math.abs(x)).toBeLessThanOrEqual(CORNER_LIMIT);
      expect(Math.abs(y)).toBeLessThanOrEqual(CORNER_LIMIT);
    }
    expect(axisError(solution, LEFT)).toBeLessThan(1e-6);
    expect(axisError(solution, RIGHT)).toBeLessThan(1e-6);
    expect(axisError(solution, steep)).toBeGreaterThan(1e-3);
  });

  it('clamps the rotation to the straighten range', () => {
    const solution = solved([
      guide('vertical', [0.2, 0.9], [0.8, 0.4]),
      guide('vertical', [0.1, 0.7], [0.7, 0.2])
    ]);
    expect(solution.angle).toBeCloseTo(-45);
  });

  it('falls back to the vertical pair when both vanishing points line up with the centre', () => {
    const solution = solved([
      LEFT,
      RIGHT,
      guide('horizontal', [0, -2.25], [1, -1.75]),
      guide('horizontal', [0, -1.75], [1, -2.25])
    ]);
    expect(solution.perspective.horizontal).toBeCloseTo(0);
    expect(axisError(solution, LEFT)).toBeLessThan(1e-6);
    expect(axisError(solution, RIGHT)).toBeLessThan(1e-6);
  });

  it('clamps an extreme vanishing point to a usable result', () => {
    const solution = solved([
      guide('vertical', [0.1, 0.9], [0.45, 0.1]),
      guide('vertical', [0.9, 0.9], [0.55, 0.1])
    ]);
    expect(Math.abs(solution.perspective.vertical)).toBeLessThanOrEqual(100);
    expect(perspectiveIsUsable(solution.perspective)).toBe(true);
    expect(Number.isFinite(solution.angle)).toBe(true);
  });
});
