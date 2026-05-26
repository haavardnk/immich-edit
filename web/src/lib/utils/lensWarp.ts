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
