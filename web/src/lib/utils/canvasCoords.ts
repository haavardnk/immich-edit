import type { Edits, LensEdits } from '$lib/types/edits';
import {
  lensWarpActive,
  lensWarpFromEdits,
  maskUvToSceneUv,
  sceneUvToMaskUv,
  type LensWarpParams
} from '$lib/utils/lensWarp';
import {
  displayUvToMaskUv,
  geometryIsIdentity,
  geometryTransformFrom,
  maskUvToDisplayUv,
  type GeometryTransform,
  type SourceDims
} from '$lib/utils/geomTransform';

export interface ViewTransform {
  geom: GeometryTransform;
  lens: LensWarpParams;
}

export function viewTransform(
  edits: Edits,
  meta: SourceDims | null,
  lens?: LensEdits
): ViewTransform {
  const sw = meta?.source_w ?? 1;
  const sh = meta?.source_h ?? 1;
  return {
    geom: geometryTransformFrom(edits.geometry, meta),
    lens: lensWarpFromEdits(lens ?? edits.lens, sw, sh)
  };
}

export function viewIsIdentity(t: ViewTransform): boolean {
  return geometryIsIdentity(t.geom) && !lensWarpActive(t.lens);
}

export function displayUvToSceneUv(t: ViewTransform, du: number, dv: number): [number, number] {
  return maskUvToSceneUv(t.lens, displayUvToMaskUv(t.geom, [du, dv]));
}

export function sceneUvToDisplayUv(t: ViewTransform, su: number, sv: number): [number, number] {
  return maskUvToDisplayUv(t.geom, sceneUvToMaskUv(t.lens, [su, sv]));
}

export function scenePerDisplayAt(t: ViewTransform, du: number, dv: number): number {
  const eps = 1e-3;
  const s0 = displayUvToSceneUv(t, du, dv);
  const sx = displayUvToSceneUv(t, du + eps, dv);
  const sy = displayUvToSceneUv(t, du, dv + eps);
  const jx = Math.hypot(sx[0] - s0[0], sx[1] - s0[1]) / eps;
  const jy = Math.hypot(sy[0] - s0[0], sy[1] - s0[1]) / eps;
  return Math.max(1e-6, (jx + jy) * 0.5);
}

export function steppedSegment(
  ax: number,
  ay: number,
  bx: number,
  by: number,
  step: number
): [number, number][] {
  const dx = bx - ax;
  const dy = by - ay;
  const n = Math.max(1, Math.ceil(Math.hypot(dx, dy) / Math.max(1e-6, step)));
  const out: [number, number][] = [];
  for (let i = 1; i <= n; i++) {
    const s = i / n;
    out.push([ax + dx * s, ay + dy * s]);
  }
  return out;
}
