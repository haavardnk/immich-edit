import {
  curvesAreIdentity,
  curvesEditsIsIdentity,
  type CurvePoint,
  type Edits
} from '$lib/types/edits';

export function encodeCurves(edits: Edits, ops: Record<string, unknown>): void {
  const curves = edits.basic.curves;
  if (curvesEditsIsIdentity(curves)) return;
  const obj: Record<string, [number, number][]> = {};
  if (!curvesAreIdentity(curves.composite))
    obj.composite = curves.composite.map((point) => [point.x, point.y]);
  if (!curvesAreIdentity(curves.r)) obj.r = curves.r.map((point) => [point.x, point.y]);
  if (!curvesAreIdentity(curves.g)) obj.g = curves.g.map((point) => [point.x, point.y]);
  if (!curvesAreIdentity(curves.b)) obj.b = curves.b.map((point) => [point.x, point.y]);
  if (!curvesAreIdentity(curves.luma)) obj.luma = curves.luma.map((point) => [point.x, point.y]);
  ops.curves = obj;
}

export function decodeCurves(ops: Record<string, unknown>, edits: Edits): void {
  const curves = ops.curves as
    | {
        points?: number[][];
        composite?: number[][];
        r?: number[][];
        g?: number[][];
        b?: number[][];
        luma?: number[][];
      }
    | undefined;
  if (!curves) return;
  const decode = (points: number[][] | undefined): CurvePoint[] | null => {
    if (!points || points.length < 2) return null;
    return points.map((point) => ({ x: point[0] ?? 0, y: point[1] ?? 0 }));
  };
  if (curves.points) {
    const legacy = decode(curves.points);
    if (legacy) edits.basic.curves.composite = legacy;
    return;
  }
  const composite = decode(curves.composite);
  if (composite) edits.basic.curves.composite = composite;
  const red = decode(curves.r);
  if (red) edits.basic.curves.r = red;
  const green = decode(curves.g);
  if (green) edits.basic.curves.g = green;
  const blue = decode(curves.b);
  if (blue) edits.basic.curves.b = blue;
  const luma = decode(curves.luma);
  if (luma) edits.basic.curves.luma = luma;
}
