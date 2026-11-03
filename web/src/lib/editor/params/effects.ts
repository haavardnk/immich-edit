import type { Edits } from '#lib/types/edits.ts';
import { field, param } from './define';
import { AMOUNT, PERCENT } from './range';
import type { ParamDef, SectionDef } from './types';

export const vignetteAmount = param({
  id: 'vignette_amount',
  label: 'Amount',
  history: 'Vignette Amount',
  synonyms: ['vignette', 'darken edges'],
  ...PERCENT,
  ...field('effects', 'vignette_amount')
});

export function noVignette(edits: Edits): boolean {
  return edits.effects.vignette_amount === 0;
}

export const vignetteMidpoint = param({
  id: 'vignette_midpoint',
  label: 'Midpoint',
  history: 'Vignette Midpoint',
  synonyms: ['vignette'],
  ...AMOUNT,
  defaultValue: 50,
  inactive: noVignette,
  ...field('effects', 'vignette_midpoint')
});

export const vignetteFeather = param({
  id: 'vignette_feather',
  label: 'Feather',
  history: 'Vignette Feather',
  synonyms: ['vignette'],
  ...AMOUNT,
  defaultValue: 50,
  inactive: noVignette,
  ...field('effects', 'vignette_feather')
});

export const vignetteRoundness = param({
  id: 'vignette_roundness',
  label: 'Roundness',
  history: 'Vignette Roundness',
  synonyms: ['vignette'],
  ...PERCENT,
  inactive: noVignette,
  ...field('effects', 'vignette_roundness')
});

export const grainAmount = param({
  id: 'grain_amount',
  label: 'Amount',
  history: 'Grain Amount',
  synonyms: ['grain', 'film', 'noise'],
  ...AMOUNT,
  ...field('effects', 'grain_amount')
});

export function noGrain(edits: Edits): boolean {
  return edits.effects.grain_amount === 0;
}

export const grainSize = param({
  id: 'grain_size',
  label: 'Size',
  history: 'Grain Size',
  synonyms: ['grain', 'film'],
  ...AMOUNT,
  defaultValue: 25,
  inactive: noGrain,
  ...field('effects', 'grain_size')
});

export const grainRoughness = param({
  id: 'grain_roughness',
  label: 'Roughness',
  history: 'Grain Roughness',
  synonyms: ['grain', 'film'],
  ...AMOUNT,
  defaultValue: 50,
  inactive: noGrain,
  ...field('effects', 'grain_roughness')
});

export const EFFECTS_PARAMS: readonly ParamDef[] = [
  vignetteAmount,
  vignetteMidpoint,
  vignetteFeather,
  vignetteRoundness,
  grainAmount,
  grainSize,
  grainRoughness
];

export const VIGNETTE: SectionDef = {
  id: 'vignette',
  title: 'Vignette',
  rows: [[vignetteAmount, vignetteMidpoint, vignetteFeather, vignetteRoundness]]
};

export const GRAIN: SectionDef = {
  id: 'grain',
  title: 'Grain',
  rows: [[grainAmount, grainSize, grainRoughness]]
};
