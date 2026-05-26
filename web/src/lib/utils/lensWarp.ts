export interface LensWarpParams {
  k1: number;
  k2: number;
  k3: number;
  zoom: number;
  width: number;
  height: number;
}

export function lensWarpActive(p: LensWarpParams): boolean {
  return !(p.k1 === 0 && p.k2 === 0 && p.k3 === 0 && p.zoom === 1);
}

interface LensLike {
  profile_enabled: boolean;
  constrain_crop: boolean;
  distortion_amount: number;
  k1: number;
  k2: number;
  k3: number;
}

function constrainZoom(k1: number, k2: number, k3: number): number {
  const s = (r: number): number => {
    const r2 = r * r;
    return 1 + k1 * r2 + k2 * r2 * r2 + k3 * r2 * r2 * r2;
  };
  if (s(1) <= 1) return 1;
  let z = 1;
  for (let i = 0; i < 32; i++) {
    const sz = s(z);
    if (sz <= 1) return 1;
    const next = 1 / sz;
    if (Math.abs(next - z) < 1e-6) return next;
    z = next;
  }
  return z;
}

export function lensWarpFromEdits(
  lens: LensLike,
  width: number,
  height: number
): LensWarpParams {
  if (!lens.profile_enabled) {
    return { k1: 0, k2: 0, k3: 0, zoom: 1, width, height };
  }
  const s = lens.distortion_amount / 100;
  const k1 = lens.k1 * s;
  const k2 = lens.k2 * s;
  const k3 = lens.k3 * s;
  const zoom = lens.constrain_crop ? constrainZoom(k1, k2, k3) : 1;
  return { k1, k2, k3, zoom, width, height };
}

function scale(p: LensWarpParams, r: number): number {
  const r2 = r * r;
  const r4 = r2 * r2;
  const r6 = r4 * r2;
  return 1 + p.k1 * r2 + p.k2 * r4 + p.k3 * r6;
}

export function maskUvToSceneUv(p: LensWarpParams, uv: [number, number]): [number, number] {
  if (!lensWarpActive(p) || p.width === 0 || p.height === 0) return [uv[0], uv[1]];
  const w = p.width;
  const h = p.height;
  const halfDiag = 0.5 * Math.sqrt(w * w + h * h);
  const nx = (uv[0] - 0.5) * w;
  const ny = (uv[1] - 0.5) * h;
  const r = (Math.sqrt(nx * nx + ny * ny) * p.zoom) / halfDiag;
  const s = scale(p, r);
  return [0.5 + (uv[0] - 0.5) * p.zoom * s, 0.5 + (uv[1] - 0.5) * p.zoom * s];
}

export function sceneUvToMaskUv(p: LensWarpParams, uv: [number, number]): [number, number] {
  if (!lensWarpActive(p) || p.width === 0 || p.height === 0) return [uv[0], uv[1]];
  const w = p.width;
  const h = p.height;
  const halfDiag = 0.5 * Math.sqrt(w * w + h * h);
  const sx = (uv[0] - 0.5) * w;
  const sy = (uv[1] - 0.5) * h;
  const lenScene = Math.sqrt(sx * sx + sy * sy);
  if (lenScene < 1e-9) return [0.5, 0.5];
  const target = lenScene / halfDiag;
  let lo = 0;
  let hi = 2;
  for (let i = 0; i < 48; i++) {
    const mid = 0.5 * (lo + hi);
    const r = mid * p.zoom;
    if (mid * p.zoom * scale(p, r) < target) lo = mid;
    else hi = mid;
  }
  const u = 0.5 * (lo + hi);
  const maskLen = u * halfDiag;
  const mx = (sx / lenScene) * maskLen;
  const my = (sy / lenScene) * maskLen;
  return [0.5 + mx / w, 0.5 + my / h];
}
