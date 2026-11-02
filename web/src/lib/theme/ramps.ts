import { hexToOklch, mix, oklchToHex } from './color';
import type { Palette, Ramp, Scheme } from './types';

export const RAMP_STEPS = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950] as const;

interface Curve {
  lightness: readonly number[];
  chroma: readonly number[];
}

interface SchemeCurves {
  towardBase: Curve;
  towardText: Curve;
}

const CURVES: Readonly<Record<Scheme, SchemeCurves>> = {
  dark: {
    towardBase: { lightness: [0.7, 0.64, 0.58, 0.51, 0.36], chroma: [0.45, 0.59, 0.71, 0.84, 1] },
    towardText: {
      lightness: [0.22, 0.4, 0.59, 0.81, 0.93],
      chroma: [0.79, 0.61, 0.43, 0.22, 0.125]
    }
  },
  light: {
    towardBase: { lightness: [0.92, 0.8, 0.59, 0.4, 0.19], chroma: [0.11, 0.22, 0.47, 0.71, 0.95] },
    towardText: {
      lightness: [0.23, 0.41, 0.63, 0.81, 0.95],
      chroma: [0.86, 0.73, 0.59, 0.45, 0.33]
    }
  }
};

function steps(anchor: string, target: string, curve: Curve): string[] {
  const from = hexToOklch(anchor);
  const to = hexToOklch(target).l;
  return curve.lightness.map((amount, index) =>
    oklchToHex({
      l: from.l + (to - from.l) * amount,
      c: from.c * (curve.chroma[index] ?? 1),
      h: from.h
    })
  );
}

export function accentRamp(anchor: string, base: string, text: string, scheme: Scheme): Ramp {
  const curves = CURVES[scheme];
  return [
    ...steps(anchor, base, curves.towardBase),
    anchor,
    ...steps(anchor, text, curves.towardText)
  ];
}

export function neutralLadder(palette: Palette): Ramp {
  return [
    palette.base,
    palette.surface0,
    palette.surface1,
    palette.surface2,
    mix(palette.surface2, palette.overlay, 0.5),
    palette.overlay,
    palette.subtext,
    mix(palette.subtext, palette.text, 0.35),
    mix(palette.subtext, palette.text, 0.65),
    mix(palette.subtext, palette.text, 0.85),
    palette.text
  ];
}
