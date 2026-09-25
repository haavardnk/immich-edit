import type { AspectLock } from '$lib/types/edits';

export const PRESET_RATIOS: ReadonlyArray<readonly [number, number]> = [
  [1, 1],
  [3, 2],
  [4, 3],
  [5, 4],
  [7, 5],
  [16, 9]
];

export const CUSTOM_KEY = 'custom';
const MAX_RATIO_SIDE = 9999;

function ratioKey(hi: number, lo: number): string {
  return `r-${hi}-${lo}`;
}

export function isPortraitAspect(aspect: AspectLock): boolean {
  return aspect.kind === 'ratio' && aspect.num < aspect.den;
}

export function aspectOptions(portrait: boolean): { value: string; label: string }[] {
  return [
    { value: 'original', label: 'Original' },
    { value: 'free', label: 'Free' },
    ...PRESET_RATIOS.map(([hi, lo]) => ({
      value: ratioKey(hi, lo),
      label: portrait ? `${lo}:${hi}` : `${hi}:${lo}`
    })),
    { value: CUSTOM_KEY, label: 'Custom…' }
  ];
}

export function selectedAspectKey(aspect: AspectLock): string {
  if (aspect.kind !== 'ratio') return aspect.kind;
  const key = ratioKey(Math.max(aspect.num, aspect.den), Math.min(aspect.num, aspect.den));
  return PRESET_RATIOS.some(([hi, lo]) => ratioKey(hi, lo) === key) ? key : CUSTOM_KEY;
}

export function aspectForKey(key: string, portrait: boolean): AspectLock | null {
  if (key === 'original' || key === 'free') return { kind: key };
  const preset = PRESET_RATIOS.find(([hi, lo]) => ratioKey(hi, lo) === key);
  if (!preset) return null;
  const [hi, lo] = preset;
  return portrait ? { kind: 'ratio', num: lo, den: hi } : { kind: 'ratio', num: hi, den: lo };
}

export function parseRatioSide(raw: string): number | null {
  const n = Number(raw.trim());
  return raw.trim() !== '' && Number.isInteger(n) && n >= 1 && n <= MAX_RATIO_SIDE ? n : null;
}
