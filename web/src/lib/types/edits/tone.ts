export interface CurvePoint {
  x: number;
  y: number;
}

export type CurveChannel = 'composite' | 'r' | 'g' | 'b' | 'luma';

export const CURVE_CHANNELS: readonly CurveChannel[] = ['composite', 'r', 'g', 'b', 'luma'];

export interface CurvesEdits {
  composite: CurvePoint[];
  r: CurvePoint[];
  g: CurvePoint[];
  b: CurvePoint[];
  luma: CurvePoint[];
}

export function identityCurve(): CurvePoint[] {
  return [
    { x: 0, y: 0 },
    { x: 1, y: 1 }
  ];
}

export function neutralCurves(): CurvesEdits {
  return {
    composite: identityCurve(),
    r: identityCurve(),
    g: identityCurve(),
    b: identityCurve(),
    luma: identityCurve()
  };
}

export function curvesAreIdentity(pts: CurvePoint[]): boolean {
  const a = pts[0];
  const b = pts[1];
  return (
    pts.length === 2 &&
    !!a &&
    !!b &&
    Math.abs(a.x) < 1e-10 &&
    Math.abs(a.y) < 1e-10 &&
    Math.abs(b.x - 1) < 1e-10 &&
    Math.abs(b.y - 1) < 1e-10
  );
}

export function curvesEditsIsIdentity(c: CurvesEdits): boolean {
  return (
    curvesAreIdentity(c.composite) &&
    curvesAreIdentity(c.r) &&
    curvesAreIdentity(c.g) &&
    curvesAreIdentity(c.b) &&
    curvesAreIdentity(c.luma)
  );
}

export interface BasicEdits {
  exposure_ev: number;
  brightness: number;
  contrast: number;
  saturation: number;
  vibrance: number;
  wb_temp: number;
  wb_tint: number;
  texture: number;
  clarity: number;
  dehaze: number;
  curves: CurvesEdits;
}

export function neutralBasic(): BasicEdits {
  return {
    exposure_ev: 0,
    brightness: 0,
    contrast: 0,
    saturation: 0,
    vibrance: 0,
    wb_temp: 0,
    wb_tint: 0,
    texture: 0,
    clarity: 0,
    dehaze: 0,
    curves: neutralCurves()
  };
}

export interface ToneEdits {
  highlights: number;
  shadows: number;
  blacks: number;
  whites: number;
}

export function neutralTone(): ToneEdits {
  return { highlights: 0, shadows: 0, blacks: 0, whites: 0 };
}
