import { clampStraighten, degToRad, radToDeg } from './geom';
import {
  clampPerspective,
  CORNER_LIMIT,
  KEYSTONE_GAIN,
  keystoneAspectMatrix,
  limitPerspective,
  mat3Apply,
  mat3Inverse,
  mat3Mul,
  neutralPerspective,
  perspectiveIsUsable,
  unitSquareCorners,
  type CornerOffsets,
  type Mat3,
  type PerspectiveEdits,
  type Point
} from './perspective';

export type GuideKind = 'vertical' | 'horizontal';

export interface UprightGuide {
  kind: GuideKind;
  from: Point;
  to: Point;
}

export interface UprightSolution {
  angle: number;
  perspective: PerspectiveEdits;
}

const MAX_GUIDES_PER_KIND = 2;
const MIN_SOLVE_GUIDES = 2;

const MIN_GUIDE_FRACTION = 0.02;
const MAX_GUIDE_TILT_DEG = 35;
const MIN_SHEAR = 1e-6;
const SHEAR_FIT_STEPS = 12;
const EPS = 1e-12;

type Vec3 = [number, number, number];

export function classifyGuide(from: Point, to: Point, diagonal: number): GuideKind | null {
  const dx = Math.abs(to[0] - from[0]);
  const dy = Math.abs(to[1] - from[1]);
  if (Math.hypot(dx, dy) < MIN_GUIDE_FRACTION * diagonal) return null;
  const tilt = radToDeg(Math.atan2(dy, dx));
  if (tilt >= 90 - MAX_GUIDE_TILT_DEG) return 'vertical';
  if (tilt <= MAX_GUIDE_TILT_DEG) return 'horizontal';
  return null;
}

export function withGuide(guides: UprightGuide[], guide: UprightGuide): UprightGuide[] {
  const family = guides.filter((g) => g.kind === guide.kind).length;
  if (family < MAX_GUIDES_PER_KIND) return [...guides, guide];
  const oldest = guides.findIndex((g) => g.kind === guide.kind);
  return [...guides.filter((_, i) => i !== oldest), guide];
}

export function solveUpright(
  guides: UprightGuide[],
  width: number,
  height: number,
  aspect: number
): UprightSolution | null {
  if (guides.length < MIN_SOLVE_GUIDES || width <= 0 || height <= 0) return null;
  const verticals = guides.filter((g) => g.kind === 'vertical');
  const horizontals = guides.filter((g) => g.kind === 'horizontal');
  const uvToPx: Mat3 = [width, 0, -width / 2, 0, height, -height / 2, 0, 0, 1];
  const [alpha, beta] = projectiveRow(
    vanishingPoint(verticals, uvToPx),
    vanishingPoint(horizontals, uvToPx)
  );
  const neutral = { ...neutralPerspective(), aspect };
  const keystone = limitPerspective(
    neutral,
    clampPerspective({
      ...neutral,
      vertical: (beta * height) / KEYSTONE_GAIN,
      horizontal: (alpha * width) / KEYSTONE_GAIN
    })
  );
  const scale: Mat3 = [width, 0, 0, 0, height, 0, 0, 0, 1];
  const unscale: Mat3 = [1 / width, 0, 0, 0, 1 / height, 0, 0, 0, 1];
  const params = mat3Mul(mat3Mul(scale, keystoneAspectMatrix(keystone)), unscale);
  const corrected = mat3Mul(params, uvToPx);
  const primaryKind: GuideKind = horizontals.length > verticals.length ? 'horizontal' : 'vertical';
  const [primary, secondary] =
    primaryKind === 'vertical' ? [verticals, horizontals] : [horizontals, verticals];
  const primaryDir = familyDirection(primary, corrected);
  if (!primaryDir) return null;
  const angle = clampStraighten(
    radToDeg(
      primaryKind === 'vertical'
        ? Math.atan(primaryDir[0] / primaryDir[1])
        : Math.atan(-primaryDir[1] / primaryDir[0])
    )
  );
  const theta = degToRad(angle);
  const secondaryDir = familyDirection(secondary, corrected);
  const shear = secondaryDir ? shearFor(primaryKind, rotate(secondaryDir, theta)) : 0;
  if (Math.abs(shear) < MIN_SHEAR) return { angle, perspective: keystone };
  return {
    angle,
    perspective: fittedShear(keystone, (t) =>
      shearCorners(primaryKind, shear * t, theta, params, uvToPx)
    )
  };
}

function fittedShear(
  keystone: PerspectiveEdits,
  cornersAt: (t: number) => CornerOffsets | null
): PerspectiveEdits {
  const at = (t: number): PerspectiveEdits | null => {
    const corners = cornersAt(t);
    if (!corners || !cornersFit(corners)) return null;
    const sheared = { ...keystone, corners };
    return perspectiveIsUsable(sheared) ? sheared : null;
  };
  const full = at(1);
  if (full) return full;
  let lo = 0;
  let hi = 1;
  let best = keystone;
  for (let i = 0; i < SHEAR_FIT_STEPS; i++) {
    const mid = (lo + hi) / 2;
    const fitted = at(mid);
    if (fitted) {
      lo = mid;
      best = fitted;
    } else {
      hi = mid;
    }
  }
  return best;
}

function cornersFit(corners: CornerOffsets): boolean {
  return corners.every((c) => Math.abs(c[0]) <= CORNER_LIMIT && Math.abs(c[1]) <= CORNER_LIMIT);
}

function vanishingPoint(family: UprightGuide[], uvToPx: Mat3): Vec3 | null {
  const [a, b] = family;
  if (!a || !b) return null;
  return normalize3(cross3(lineThrough(a, uvToPx), lineThrough(b, uvToPx)));
}

function lineThrough(guide: UprightGuide, uvToPx: Mat3): Vec3 {
  const [x0, y0] = mat3Apply(uvToPx, guide.from);
  const [x1, y1] = mat3Apply(uvToPx, guide.to);
  return normalize3(cross3([x0, y0, 1], [x1, y1, 1])) ?? [0, 0, 0];
}

function projectiveRow(vertical: Vec3 | null, horizontal: Vec3 | null): [number, number] {
  if (vertical && horizontal) {
    const det = vertical[0] * horizontal[1] - vertical[1] * horizontal[0];
    if (Math.abs(det) > EPS) {
      return [
        (horizontal[2] * vertical[1] - vertical[2] * horizontal[1]) / det,
        (vertical[2] * horizontal[0] - horizontal[2] * vertical[0]) / det
      ];
    }
  }
  if (vertical) return [0, axisRow(vertical[2], vertical[1])];
  if (horizontal) return [axisRow(horizontal[2], horizontal[0]), 0];
  return [0, 0];
}

function axisRow(w: number, along: number): number {
  return Math.abs(along) < EPS ? 0 : -w / along;
}

function familyDirection(family: UprightGuide[], map: Mat3): Point | null {
  const sum = family.reduce<Point>(
    (acc, g) => {
      const d = normalize2(sub(mat3Apply(map, g.to), mat3Apply(map, g.from)));
      if (!d) return acc;
      const flip = acc[0] * d[0] + acc[1] * d[1] < 0 ? -1 : 1;
      return [acc[0] + flip * d[0], acc[1] + flip * d[1]];
    },
    [0, 0]
  );
  return normalize2(sum);
}

function shearFor(kind: GuideKind, e: Point): number {
  if (kind === 'vertical') return Math.abs(e[0]) < EPS ? 0 : -e[1] / e[0];
  return Math.abs(e[1]) < EPS ? 0 : -e[0] / e[1];
}

function shearCorners(
  kind: GuideKind,
  shear: number,
  theta: number,
  params: Mat3,
  uvToPx: Mat3
): CornerOffsets | null {
  const paramsInv = mat3Inverse(params);
  const pxToUv = mat3Inverse(uvToPx);
  if (!paramsInv || !pxToUv) return null;
  const c = Math.cos(theta);
  const s = Math.sin(theta);
  const rot: Mat3 = [c, -s, 0, s, c, 0, 0, 0, 1];
  const unrot: Mat3 = [c, s, 0, -s, c, 0, 0, 0, 1];
  const skew: Mat3 =
    kind === 'vertical' ? [1, 0, 0, shear, 1, 0, 0, 0, 1] : [1, shear, 0, 0, 1, 0, 0, 0, 1];
  const local = mat3Mul(mat3Mul(unrot, skew), rot);
  const px = mat3Mul(mat3Mul(paramsInv, local), params);
  const uv = mat3Mul(mat3Mul(pxToUv, px), uvToPx);
  const offset = (corner: Point): Point => sub(mat3Apply(uv, corner), corner);
  const [p0, p1, p2, p3] = unitSquareCorners();
  return [offset(p0), offset(p1), offset(p2), offset(p3)];
}

function rotate(p: Point, theta: number): Point {
  const c = Math.cos(theta);
  const s = Math.sin(theta);
  return [p[0] * c - p[1] * s, p[0] * s + p[1] * c];
}

function cross3(a: Vec3, b: Vec3): Vec3 {
  return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
}

function normalize3(v: Vec3): Vec3 | null {
  const n = Math.hypot(v[0], v[1], v[2]);
  return n < EPS ? null : [v[0] / n, v[1] / n, v[2] / n];
}

function normalize2(v: Point): Point | null {
  const n = Math.hypot(v[0], v[1]);
  return n < EPS ? null : [v[0] / n, v[1] / n];
}

function sub(a: Point, b: Point): Point {
  return [a[0] - b[0], a[1] - b[1]];
}
