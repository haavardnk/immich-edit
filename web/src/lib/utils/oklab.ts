export type Rgb = readonly [number, number, number];

export interface Oklab {
  l: number;
  a: number;
  b: number;
}

export function toLinear(channel: number): number {
  return channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
}

export function toGamma(channel: number): number {
  return channel <= 0.0031308 ? 12.92 * channel : 1.055 * channel ** (1 / 2.4) - 0.055;
}

export function linearToOklab([r, g, b]: Rgb): Oklab {
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  return {
    l: 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    a: 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    b: 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s
  };
}

export function oklabToLinear({ l, a, b }: Oklab): Rgb {
  const lc = (l + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const mc = (l - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const sc = (l - 0.0894841775 * a - 1.291485548 * b) ** 3;
  return [
    4.0767416621 * lc - 3.3077115913 * mc + 0.2309699292 * sc,
    -1.2684380046 * lc + 2.6097574011 * mc - 0.3413193965 * sc,
    -0.0041960863 * lc - 0.7034186147 * mc + 1.707614701 * sc
  ];
}

export function displayToOklab([r, g, b]: Rgb): Oklab {
  return linearToOklab([toLinear(r), toLinear(g), toLinear(b)]);
}

export function oklabToDisplay(lab: Oklab): [number, number, number] {
  const unit = (channel: number): number => Math.min(1, Math.max(0, toGamma(Math.max(0, channel))));
  const [r, g, b] = oklabToLinear(lab);
  return [unit(r), unit(g), unit(b)];
}

export function oklabDistance(p: Oklab, q: Oklab): number {
  return Math.hypot(p.l - q.l, p.a - q.a, p.b - q.b);
}
