export type Mat3 = [number, number, number, number, number, number, number, number, number];

export type CornerOffsets = [[number, number], [number, number], [number, number], [number, number]];

export interface PerspectiveEdits {
  vertical: number;
  horizontal: number;
  aspect: number;
  corners: CornerOffsets | null;
}

export const IDENTITY_MAT3: Mat3 = [1, 0, 0, 0, 1, 0, 0, 0, 1];

export const CORNER_LIMIT = 0.25;

const KEYSTONE_GAIN = 0.005;
const ASPECT_BASE = 1.5;
const MIN_W = 0.05;
const MIN_QUAD_AREA = 0.05;

export function neutralPerspective(): PerspectiveEdits {
  return {
    vertical: 0,
    horizontal: 0,
    aspect: 0,
    corners: null
  };
}

export function perspectiveIsIdentity(p: PerspectiveEdits | null): boolean {
  if (!p) return true;
  return (
    Math.abs(p.vertical) < 1e-4 &&
    Math.abs(p.horizontal) < 1e-4 &&
    Math.abs(p.aspect) < 1e-4 &&
    (p.corners === null || p.corners.every((c) => Math.abs(c[0]) < 1e-6 && Math.abs(c[1]) < 1e-6))
  );
}

export function clampPerspective(p: PerspectiveEdits): PerspectiveEdits {
  return {
    vertical: clamp(p.vertical, -100, 100),
    horizontal: clamp(p.horizontal, -100, 100),
    aspect: clamp(p.aspect, -100, 100),
    corners: p.corners ? clampCorners(p.corners) : null
  };
}

export function perspectiveForward(p: PerspectiveEdits | null): Mat3 {
  return perspectiveMatrices(p)[0];
}

export function perspectiveInverse(p: PerspectiveEdits | null): Mat3 {
  return perspectiveMatrices(p)[1];
}

export function perspectiveQuad(p: PerspectiveEdits | null): [number, number][] {
  const f = perspectiveForward(p);
  return unitSquareCorners().map((c) => mat3Apply(f, c));
}

function perspectiveParamsMatrix(p: PerspectiveEdits): Mat3 {
  const c = clampPerspective(p);
  const raw = mat3Mul(centeredToUv(paramsMatrix(c)), cornerMatrix(c));
  return mat3Mul(fitToFrame(raw), centeredToUv(paramsMatrix(c)));
}

export function cornerOffsetsFor(
  p: PerspectiveEdits,
  index: number,
  targetUv: [number, number]
): CornerOffsets {
  const base = unitSquareCorners()[index];
  let next = p.corners ?? emptyCorners();
  for (let i = 0; i < 4; i++) {
    const inv = mat3Inverse(perspectiveParamsMatrix({ ...p, corners: next }));
    if (!inv) return next;
    const quadPoint = mat3Apply(inv, targetUv);
    const candidate: CornerOffsets = [next[0], next[1], next[2], next[3]];
    candidate[index] = [quadPoint[0] - base[0], quadPoint[1] - base[1]];
    next = clampCorners(candidate);
  }
  return next;
}

function emptyCorners(): CornerOffsets {
  return [
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0]
  ];
}

export function perspectiveCssMatrix(m: Mat3, w: number, h: number): string {
  const sw = Math.max(w, 1e-6);
  const sh = Math.max(h, 1e-6);
  const toUv: Mat3 = [1 / sw, 0, 0.5, 0, 1 / sh, 0.5, 0, 0, 1];
  const toPx: Mat3 = [sw, 0, -sw * 0.5, 0, sh, -sh * 0.5, 0, 0, 1];
  const a = mat3Mul(mat3Mul(toPx, m), toUv);
  return `matrix3d(${a[0]}, ${a[3]}, 0, ${a[6]}, ${a[1]}, ${a[4]}, 0, ${a[7]}, 0, 0, 1, 0, ${a[2]}, ${a[5]}, 0, ${a[8]})`;
}

export function mat3Mul(a: Mat3, b: Mat3): Mat3 {
  const out = new Array<number>(9);
  for (let r = 0; r < 3; r++) {
    for (let c = 0; c < 3; c++) {
      out[r * 3 + c] = a[r * 3] * b[c] + a[r * 3 + 1] * b[3 + c] + a[r * 3 + 2] * b[6 + c];
    }
  }
  return out as Mat3;
}

export function mat3Inverse(m: Mat3): Mat3 | null {
  const c00 = m[4] * m[8] - m[5] * m[7];
  const c01 = m[5] * m[6] - m[3] * m[8];
  const c02 = m[3] * m[7] - m[4] * m[6];
  const det = m[0] * c00 + m[1] * c01 + m[2] * c02;
  if (Math.abs(det) < 1e-12) return null;
  const d = 1 / det;
  return [
    c00 * d,
    (m[2] * m[7] - m[1] * m[8]) * d,
    (m[1] * m[5] - m[2] * m[4]) * d,
    c01 * d,
    (m[0] * m[8] - m[2] * m[6]) * d,
    (m[2] * m[3] - m[0] * m[5]) * d,
    c02 * d,
    (m[1] * m[6] - m[0] * m[7]) * d,
    (m[0] * m[4] - m[1] * m[3]) * d
  ];
}

export function mat3Apply(m: Mat3, p: [number, number]): [number, number] {
  const w = m[6] * p[0] + m[7] * p[1] + m[8];
  if (Math.abs(w) < 1e-9) return [p[0], p[1]];
  return [
    (m[0] * p[0] + m[1] * p[1] + m[2]) / w,
    (m[3] * p[0] + m[4] * p[1] + m[5]) / w
  ];
}

export function squareToQuad(quad: [number, number][]): Mat3 | null {
  const [p0, p1, p2, p3] = quad;
  const sx = p0[0] - p1[0] + p2[0] - p3[0];
  const sy = p0[1] - p1[1] + p2[1] - p3[1];
  if (Math.abs(sx) < 1e-9 && Math.abs(sy) < 1e-9) {
    return [p1[0] - p0[0], p3[0] - p0[0], p0[0], p1[1] - p0[1], p3[1] - p0[1], p0[1], 0, 0, 1];
  }
  const dx1 = p1[0] - p2[0];
  const dx2 = p3[0] - p2[0];
  const dy1 = p1[1] - p2[1];
  const dy2 = p3[1] - p2[1];
  const den = dx1 * dy2 - dx2 * dy1;
  if (Math.abs(den) < 1e-12) return null;
  const g = (sx * dy2 - dx2 * sy) / den;
  const h = (dx1 * sy - sx * dy1) / den;
  return [
    p1[0] - p0[0] + g * p1[0],
    p3[0] - p0[0] + h * p3[0],
    p0[0],
    p1[1] - p0[1] + g * p1[1],
    p3[1] - p0[1] + h * p3[1],
    p0[1],
    g,
    h,
    1
  ];
}

function perspectiveMatrices(p: PerspectiveEdits | null): [Mat3, Mat3] {
  if (!p || perspectiveIsIdentity(p)) return [IDENTITY_MAT3, IDENTITY_MAT3];
  return tryMatrices(clampPerspective(p)) ?? [IDENTITY_MAT3, IDENTITY_MAT3];
}

function tryMatrices(c: PerspectiveEdits): [Mat3, Mat3] | null {
  const raw = mat3Mul(centeredToUv(paramsMatrix(c)), cornerMatrix(c));
  const forward = mat3Mul(fitToFrame(raw), raw);
  const inverse = mat3Inverse(forward);
  if (!inverse) return null;
  const degenerate = unitSquareCorners().some(
    (q) => Math.abs(inverse[6] * q[0] + inverse[7] * q[1] + inverse[8]) < MIN_W
  );
  if (degenerate) return null;
  return [forward, inverse];
}

function fitToFrame(m: Mat3): Mat3 {
  const pts = unitSquareCorners().map((p) => mat3Apply(m, p));
  const xs = pts.map((p) => p[0]);
  const ys = pts.map((p) => p[1]);
  const minX = Math.min(...xs);
  const maxX = Math.max(...xs);
  const minY = Math.min(...ys);
  const maxY = Math.max(...ys);
  const w = Math.max(maxX - minX, 1e-6);
  const h = Math.max(maxY - minY, 1e-6);
  const s = Math.min(1 / w, 1 / h, 1);
  const cx = (minX + maxX) * 0.5;
  const cy = (minY + maxY) * 0.5;
  return [s, 0, 0.5 - s * cx, 0, s, 0.5 - s * cy, 0, 0, 1];
}

export function perspectiveIsUsable(p: PerspectiveEdits): boolean {
  return perspectiveIsIdentity(p) || tryMatrices(clampPerspective(p)) !== null;
}

export function limitPerspective(from: PerspectiveEdits, to: PerspectiveEdits): PerspectiveEdits {
  if (perspectiveIsUsable(to)) return to;
  let lo = 0;
  let hi = 1;
  for (let i = 0; i < 12; i++) {
    const mid = (lo + hi) / 2;
    if (perspectiveIsUsable(lerpPerspective(from, to, mid))) lo = mid;
    else hi = mid;
  }
  return lerpPerspective(from, to, lo);
}

function lerpPerspective(a: PerspectiveEdits, b: PerspectiveEdits, t: number): PerspectiveEdits {
  const ca = a.corners ?? emptyCorners();
  const cb = b.corners ?? emptyCorners();
  return {
    vertical: lerp(a.vertical, b.vertical, t),
    horizontal: lerp(a.horizontal, b.horizontal, t),
    aspect: lerp(a.aspect, b.aspect, t),
    corners:
      a.corners || b.corners
        ? [
            lerpPoint(ca[0], cb[0], t),
            lerpPoint(ca[1], cb[1], t),
            lerpPoint(ca[2], cb[2], t),
            lerpPoint(ca[3], cb[3], t)
          ]
        : null
  };
}

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

function lerpPoint(a: [number, number], b: [number, number], t: number): [number, number] {
  return [lerp(a[0], b[0], t), lerp(a[1], b[1], t)];
}

function paramsMatrix(p: PerspectiveEdits): Mat3 {
  const kv = p.vertical * KEYSTONE_GAIN;
  const kh = p.horizontal * KEYSTONE_GAIN;
  const a = Math.pow(ASPECT_BASE, p.aspect / 100);
  const keystoneV: Mat3 = [1, 0, 0, 0, 1, 0, 0, kv, 1];
  const keystoneH: Mat3 = [1, 0, 0, 0, 1, 0, kh, 0, 1];
  const aspect: Mat3 = [a, 0, 0, 0, 1 / a, 0, 0, 0, 1];
  return mat3Mul(mat3Mul(aspect, keystoneV), keystoneH);
}

function cornerMatrix(p: PerspectiveEdits): Mat3 {
  if (!p.corners) return IDENTITY_MAT3;
  const base = unitSquareCorners();
  const quad = base.map((b, i) => [b[0] + p.corners![i][0], b[1] + p.corners![i][1]] as [number, number]);
  if (!quadIsUsable(quad)) return IDENTITY_MAT3;
  return squareToQuad(quad) ?? IDENTITY_MAT3;
}

function quadIsUsable(quad: [number, number][]): boolean {
  let area = 0;
  let sign = 0;
  for (let i = 0; i < 4; i++) {
    const a = quad[i];
    const b = quad[(i + 1) % 4];
    const c = quad[(i + 2) % 4];
    area += a[0] * b[1] - b[0] * a[1];
    const cross = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]);
    if (sign === 0) sign = cross;
    else if (cross * sign < 0) return false;
  }
  return area * 0.5 >= MIN_QUAD_AREA;
}

function clampCorners(corners: CornerOffsets): CornerOffsets {
  return corners.map((c) => [
    clamp(c[0], -CORNER_LIMIT, CORNER_LIMIT),
    clamp(c[1], -CORNER_LIMIT, CORNER_LIMIT)
  ]) as CornerOffsets;
}

function unitSquareCorners(): [number, number][] {
  return [
    [0, 0],
    [1, 0],
    [1, 1],
    [0, 1]
  ];
}

function centeredToUv(m: Mat3): Mat3 {
  const toUv: Mat3 = [1, 0, 0.5, 0, 1, 0.5, 0, 0, 1];
  const toCentered: Mat3 = [1, 0, -0.5, 0, 1, -0.5, 0, 0, 1];
  return mat3Mul(mat3Mul(toUv, m), toCentered);
}

function clamp(v: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, v));
}
