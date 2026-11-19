import {
  linearToOklab,
  oklabToLinear,
  toGamma,
  toLinear,
  type Oklab,
  type Rgb
} from '#lib/utils/oklab.ts';

export interface Oklch {
  l: number;
  c: number;
  h: number;
}

const HEX = /^#[0-9a-f]{6}$/i;

const GAMUT_EPSILON = 1e-4;

function isHex(value: string): boolean {
  return HEX.test(value);
}

function hexToRgb(hex: string): Rgb {
  if (!isHex(hex)) throw new Error(`not a #rrggbb colour: ${hex}`);
  const value = Number.parseInt(hex.slice(1), 16);
  return [((value >> 16) & 255) / 255, ((value >> 8) & 255) / 255, (value & 255) / 255];
}

function channelHex(channel: number): string {
  const byte = Math.round(Math.min(1, Math.max(0, channel)) * 255);
  return byte.toString(16).padStart(2, '0');
}

function rgbToHex([r, g, b]: Rgb): string {
  return `#${channelHex(r)}${channelHex(g)}${channelHex(b)}`;
}

function hexToOklab(hex: string): Oklab {
  const [r, g, b] = hexToRgb(hex);
  return linearToOklab([toLinear(r), toLinear(g), toLinear(b)]);
}

function inGamut(rgb: Rgb): boolean {
  return rgb.every((channel) => channel >= -GAMUT_EPSILON && channel <= 1 + GAMUT_EPSILON);
}

function oklabToHex(lab: Oklab): string {
  const [r, g, b] = oklabToLinear(lab);
  return rgbToHex([toGamma(Math.max(0, r)), toGamma(Math.max(0, g)), toGamma(Math.max(0, b))]);
}

function oklchToOklab({ l, c, h }: Oklch): Oklab {
  const radians = (h * Math.PI) / 180;
  return { l, a: c * Math.cos(radians), b: c * Math.sin(radians) };
}

export function hexToOklch(hex: string): Oklch {
  const { l, a, b } = hexToOklab(hex);
  const c = Math.hypot(a, b);
  const h = c < 1e-6 ? 0 : ((Math.atan2(b, a) * 180) / Math.PI + 360) % 360;
  return { l, c, h };
}

export function oklchToHex(color: Oklch): string {
  const l = Math.min(1, Math.max(0, color.l));
  const fits = (c: number): boolean => inGamut(oklabToLinear(oklchToOklab({ l, c, h: color.h })));
  let low = 0;
  let high = Math.max(0, color.c);
  if (fits(high)) return oklabToHex(oklchToOklab({ l, c: high, h: color.h }));
  for (let step = 0; step < 24; step += 1) {
    const mid = (low + high) / 2;
    if (fits(mid)) low = mid;
    else high = mid;
  }
  return oklabToHex(oklchToOklab({ l, c: low, h: color.h }));
}

export function mix(from: string, to: string, amount: number): string {
  const a = hexToOklab(from);
  const b = hexToOklab(to);
  return oklabToHex({
    l: a.l + (b.l - a.l) * amount,
    a: a.a + (b.a - a.a) * amount,
    b: a.b + (b.b - a.b) * amount
  });
}

export function shiftLightness(hex: string, delta: number): string {
  const color = hexToOklch(hex);
  return oklchToHex({ ...color, l: color.l + delta });
}

export function composite(foreground: string, background: string, alpha: number): string {
  const top = hexToRgb(foreground);
  const bottom = hexToRgb(background);
  return rgbToHex([
    top[0] * alpha + bottom[0] * (1 - alpha),
    top[1] * alpha + bottom[1] * (1 - alpha),
    top[2] * alpha + bottom[2] * (1 - alpha)
  ]);
}

function relativeLuminance(hex: string): number {
  const [r, g, b] = hexToRgb(hex);
  return 0.2126 * toLinear(r) + 0.7152 * toLinear(g) + 0.0722 * toLinear(b);
}

export function contrastRatio(a: string, b: string): number {
  const first = relativeLuminance(a);
  const second = relativeLuminance(b);
  return (Math.max(first, second) + 0.05) / (Math.min(first, second) + 0.05);
}
