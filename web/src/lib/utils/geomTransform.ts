import type { CropRect } from '../types/edits';
import { rotatedBbox, degToRad, type Size } from './geom';

export type RotateQuarter = 0 | 90 | 180 | 270;

export interface GeometryTransform {
  inputW: number;
  inputH: number;
  rotateQuarter: RotateQuarter;
  flipH: boolean;
  flipV: boolean;
  angleDeg: number;
  crop: CropRect;
  outputW: number;
  outputH: number;
}

export function geometryIsIdentity(t: GeometryTransform): boolean {
  return (
    t.rotateQuarter === 0 &&
    !t.flipH &&
    !t.flipV &&
    Math.abs(t.angleDeg) < 1e-4 &&
    t.crop.x === 0 &&
    t.crop.y === 0 &&
    t.crop.w === 1 &&
    t.crop.h === 1
  );
}

export function geometryOrientedSize(t: GeometryTransform): { w: number; h: number } {
  if (t.rotateQuarter === 90 || t.rotateQuarter === 270) {
    return { w: t.inputH, h: t.inputW };
  }
  return { w: t.inputW, h: t.inputH };
}

export function geometryBbox(t: GeometryTransform): Size {
  const o = geometryOrientedSize(t);
  return rotatedBbox(o.w, o.h, t.angleDeg);
}

function orthoForward(
  rot: RotateQuarter,
  flipH: boolean,
  flipV: boolean,
  mu: number,
  mv: number
): [number, number] {
  let u: number;
  let v: number;
  switch (rot) {
    case 90:
      u = 1 - mv;
      v = mu;
      break;
    case 180:
      u = 1 - mu;
      v = 1 - mv;
      break;
    case 270:
      u = mv;
      v = 1 - mu;
      break;
    default:
      u = mu;
      v = mv;
  }
  if (flipH) u = 1 - u;
  if (flipV) v = 1 - v;
  return [u, v];
}

function orthoInverse(
  rot: RotateQuarter,
  flipH: boolean,
  flipV: boolean,
  u: number,
  v: number
): [number, number] {
  let uu = u;
  let vv = v;
  if (flipH) uu = 1 - uu;
  if (flipV) vv = 1 - vv;
  switch (rot) {
    case 90:
      return [vv, 1 - uu];
    case 180:
      return [1 - uu, 1 - vv];
    case 270:
      return [1 - vv, uu];
    default:
      return [uu, vv];
  }
}

export function displayUvToMaskUv(
  t: GeometryTransform,
  uv: [number, number]
): [number, number] {
  if (geometryIsIdentity(t)) return [uv[0], uv[1]];
  const o = geometryOrientedSize(t);
  const bbox = geometryBbox(t);
  const a = degToRad(t.angleDeg);
  const cosA = Math.cos(a);
  const sinA = Math.sin(a);
  const bxRel = t.crop.x + uv[0] * t.crop.w;
  const byRel = t.crop.y + uv[1] * t.crop.h;
  const cxPx = (bxRel - 0.5) * bbox.w;
  const cyPx = (byRel - 0.5) * bbox.h;
  const sxPx = cxPx * cosA + cyPx * sinA;
  const syPx = -cxPx * sinA + cyPx * cosA;
  const uO = sxPx / o.w + 0.5;
  const vO = syPx / o.h + 0.5;
  return orthoInverse(t.rotateQuarter, t.flipH, t.flipV, uO, vO);
}

export function maskUvToDisplayUv(
  t: GeometryTransform,
  uv: [number, number]
): [number, number] {
  if (geometryIsIdentity(t)) return [uv[0], uv[1]];
  const o = geometryOrientedSize(t);
  const bbox = geometryBbox(t);
  const a = degToRad(t.angleDeg);
  const cosA = Math.cos(a);
  const sinA = Math.sin(a);
  const [uO, vO] = orthoForward(t.rotateQuarter, t.flipH, t.flipV, uv[0], uv[1]);
  const sxPx = (uO - 0.5) * o.w;
  const syPx = (vO - 0.5) * o.h;
  const cxPx = sxPx * cosA - syPx * sinA;
  const cyPx = sxPx * sinA + syPx * cosA;
  const bxRel = cxPx / bbox.w + 0.5;
  const byRel = cyPx / bbox.h + 0.5;
  const cw = Math.max(t.crop.w, 1e-9);
  const ch = Math.max(t.crop.h, 1e-9);
  return [(bxRel - t.crop.x) / cw, (byRel - t.crop.y) / ch];
}
