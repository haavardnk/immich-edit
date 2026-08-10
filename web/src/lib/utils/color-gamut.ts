import type { ColorSpaceOpt } from '$lib/api/export';

export function displayGamutIsWide(): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return false;
  return window.matchMedia('(color-gamut: p3)').matches;
}

export function previewColorSpace(
  proofSpace: ColorSpaceOpt,
  gamutWarn: boolean,
  wideGamut: boolean
): ColorSpaceOpt {
  if (proofSpace !== 'srgb' || gamutWarn) return proofSpace;
  return wideGamut ? 'displayp3' : 'srgb';
}
