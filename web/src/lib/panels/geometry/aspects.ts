import type { CropRatio } from '#lib/api/preferences.ts';
import type { AspectLock } from '#lib/types/edits.ts';

export const CHIP_RATIOS: readonly CropRatio[] = [
  { num: 1, den: 1 },
  { num: 3, den: 2 },
  { num: 4, den: 3 },
  { num: 16, den: 9 }
];

export const MORE_RATIOS: readonly CropRatio[] = [
  { num: 5, den: 4 },
  { num: 7, den: 5 }
];

export const PRESET_RATIOS: readonly CropRatio[] = [
  { num: 1, den: 1 },
  { num: 3, den: 2 },
  { num: 4, den: 3 },
  { num: 5, den: 4 },
  { num: 7, den: 5 },
  { num: 16, den: 9 }
];

const MAX_RATIO_SIDE = 9999;
const MAX_DECIMALS = 3;
const MAX_REDUCED_SIDE = 9;
const RATIO_PATTERN = /^(\d+(?:\.\d+)?)\s*(?:[:x×/]\s*(\d+(?:\.\d+)?))?$/i;

export function longSideFirst(ratio: CropRatio): CropRatio {
  return ratio.num < ratio.den ? { num: ratio.den, den: ratio.num } : ratio;
}

export function ratioKey(ratio: CropRatio): string {
  const { num, den } = longSideFirst(ratio);
  return `r-${num}-${den}`;
}

export function aspectKey(aspect: AspectLock): string {
  return aspect.kind === 'ratio' ? ratioKey(aspect) : aspect.kind;
}

export function isPortraitAspect(aspect: AspectLock): boolean {
  return aspect.kind === 'ratio' && aspect.num < aspect.den;
}

export function orientedAspect(ratio: CropRatio, portrait: boolean): AspectLock {
  const { num, den } = longSideFirst(ratio);
  return portrait ? { kind: 'ratio', num: den, den: num } : { kind: 'ratio', num, den };
}

function isDecimalScale(side: number): boolean {
  return side > 1 && /^10+$/.test(String(side));
}

function sideLabels(ratio: CropRatio): [string, string] {
  const { num, den } = ratio;
  if (num >= den && isDecimalScale(den)) return [String(num / den), '1'];
  if (den > num && isDecimalScale(num)) return ['1', String(den / num)];
  return [String(num), String(den)];
}

export function ratioLabel(ratio: CropRatio, portrait = false): string {
  const [long, short] = sideLabels(longSideFirst(ratio));
  return portrait ? `${short}:${long}` : `${long}:${short}`;
}

export function aspectText(aspect: AspectLock): string {
  if (aspect.kind !== 'ratio') return '';
  const [first, second] = sideLabels(aspect);
  return `${first}:${second}`;
}

export function isPresetRatio(ratio: CropRatio): boolean {
  const key = ratioKey(ratio);
  return PRESET_RATIOS.some((preset) => ratioKey(preset) === key);
}

export function ratioReadout(ratio: CropRatio): string {
  const { num, den } = longSideFirst(ratio);
  return `${Number((num / den).toFixed(2))} : 1`;
}

function greatestDivisor(a: number, b: number): number {
  return b === 0 ? a : greatestDivisor(b, a % b);
}

function decimals(raw: string): number {
  const dot = raw.indexOf('.');
  return dot < 0 ? 0 : raw.length - dot - 1;
}

export function parseRatio(text: string): CropRatio | null {
  const match = RATIO_PATTERN.exec(text.trim());
  if (!match) return null;
  const first = match[1] ?? '';
  const second = match[2] ?? '1';
  const places = Math.max(decimals(first), decimals(second));
  if (places > MAX_DECIMALS) return null;
  const scale = 10 ** places;
  const num = Math.round(Number(first) * scale);
  const den = Math.round(Number(second) * scale);
  if (num < 1 || den < 1) return null;
  const divisor = places > 0 ? greatestDivisor(num, den) : 1;
  const reduced = Math.min(num, den) / divisor <= MAX_REDUCED_SIDE;
  const ratio = reduced ? { num: num / divisor, den: den / divisor } : { num, den };
  return ratio.num <= MAX_RATIO_SIDE && ratio.den <= MAX_RATIO_SIDE ? ratio : null;
}

export function parseRatioSide(raw: string): number | null {
  const n = Number(raw.trim());
  return raw.trim() !== '' && Number.isInteger(n) && n >= 1 && n <= MAX_RATIO_SIDE ? n : null;
}
