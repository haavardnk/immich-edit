import {
  AI_DENOISE_STRENGTH,
  NEUTRAL_DETAIL,
  neutralSharpenAmount,
  type Edits
} from '#lib/types/edits.ts';
import { field, param } from './define';
import { AMOUNT, amountRange, tenths } from './range';
import type { ParamDef, SectionDef } from './types';

export const captureCornerBoost = param({
  id: 'capture_corner_boost',
  label: 'Corner Boost',
  history: 'Capture Corner Boost',
  synonyms: ['capture sharpening', 'edges'],
  ...AMOUNT,
  inactive: (edits, isRaw) => !isRaw || !edits.detail.capture_sharpen,
  ...field('detail', 'capture_corner_boost')
});

export const sharpenAmount = param({
  id: 'sharpen_amount',
  label: 'Amount',
  history: 'Sharpen Amount',
  synonyms: ['sharpening', 'sharpness'],
  ...amountRange(150),
  defaultValue: neutralSharpenAmount(false),
  rawDefaultValue: neutralSharpenAmount(true),
  get: (edits, isRaw) => edits.detail.sharpen_amount ?? neutralSharpenAmount(isRaw),
  set: (edits, value, isRaw) => {
    edits.detail.sharpen_amount = value === neutralSharpenAmount(isRaw) ? null : value;
  }
});

function unsharpened(edits: Edits, isRaw: boolean): boolean {
  return sharpenAmount.get(edits, isRaw) === 0;
}

export const sharpenRadius = param({
  id: 'sharpen_radius',
  label: 'Radius',
  history: 'Sharpen Radius',
  synonyms: ['sharpening'],
  min: 0.5,
  max: 3,
  step: 0.1,
  fineStep: 0.1,
  format: tenths,
  defaultValue: 1,
  inactive: unsharpened,
  previewMode: 'sharpen_radius',
  ...field('detail', 'sharpen_radius')
});

export const sharpenDetail = param({
  id: 'sharpen_detail',
  label: 'Detail',
  history: 'Sharpen Detail',
  synonyms: ['sharpening'],
  ...AMOUNT,
  defaultValue: 25,
  inactive: unsharpened,
  previewMode: 'sharpen_detail',
  ...field('detail', 'sharpen_detail')
});

export const sharpenMasking = param({
  id: 'sharpen_masking',
  label: 'Masking',
  history: 'Sharpen Masking',
  synonyms: ['sharpening', 'edge mask'],
  ...AMOUNT,
  inactive: unsharpened,
  previewMode: 'sharpen_mask',
  ...field('detail', 'sharpen_masking')
});

function noAiDenoise(edits: Edits, isRaw: boolean): boolean {
  return !isRaw || !edits.detail.ai_denoise;
}

export const aiDenoiseStrength = param({
  id: 'ai_denoise_strength',
  label: 'Strength',
  history: 'AI Denoise Strength',
  synonyms: ['AI denoise', 'noise'],
  ...AMOUNT,
  defaultValue: AI_DENOISE_STRENGTH,
  inactive: noAiDenoise,
  ...field('detail', 'ai_denoise_strength')
});

export const aiDenoiseDetail = param({
  id: 'ai_denoise_detail',
  label: 'Detail',
  history: 'AI Denoise Detail',
  synonyms: ['AI denoise', 'noise'],
  ...AMOUNT,
  inactive: noAiDenoise,
  ...field('detail', 'ai_denoise_detail')
});

export const lumaNr = param({
  id: 'luma_nr_amount',
  label: 'Luminance',
  history: 'Luminance NR',
  synonyms: ['noise', 'NR', 'denoise'],
  ...AMOUNT,
  ...field('detail', 'luma_nr_amount')
});

export function noLumaNr(edits: Edits): boolean {
  return edits.detail.luma_nr_amount === 0;
}

export const lumaNrDetail = param({
  id: 'luma_nr_detail',
  label: 'Detail',
  history: 'Luminance NR Detail',
  synonyms: ['noise', 'NR'],
  ...AMOUNT,
  defaultValue: 50,
  inactive: noLumaNr,
  ...field('detail', 'luma_nr_detail')
});

export const lumaNrContrast = param({
  id: 'luma_nr_contrast',
  label: 'Contrast',
  history: 'Luminance NR Contrast',
  synonyms: ['noise', 'NR'],
  ...AMOUNT,
  inactive: noLumaNr,
  ...field('detail', 'luma_nr_contrast')
});

export const colorNr = param({
  id: 'color_nr_amount',
  label: 'Color',
  history: 'Color NR',
  synonyms: ['noise', 'NR', 'chroma noise'],
  ...AMOUNT,
  ...field('detail', 'color_nr_amount')
});

export function noColorNr(edits: Edits): boolean {
  return edits.detail.color_nr_amount === 0;
}

export const colorNrDetail = param({
  id: 'color_nr_detail',
  label: 'Detail',
  history: 'Color NR Detail',
  synonyms: ['noise', 'NR'],
  ...AMOUNT,
  defaultValue: 50,
  inactive: noColorNr,
  ...field('detail', 'color_nr_detail')
});

export const colorNrSmoothness = param({
  id: 'color_nr_smoothness',
  label: 'Smoothness',
  history: 'Color NR Smoothness',
  synonyms: ['noise', 'NR'],
  ...AMOUNT,
  defaultValue: 50,
  inactive: noColorNr,
  ...field('detail', 'color_nr_smoothness')
});

export const DETAIL_PARAMS: readonly ParamDef[] = [
  captureCornerBoost,
  sharpenAmount,
  sharpenRadius,
  sharpenDetail,
  sharpenMasking,
  aiDenoiseStrength,
  aiDenoiseDetail,
  lumaNr,
  lumaNrDetail,
  lumaNrContrast,
  colorNr,
  colorNrDetail,
  colorNrSmoothness
];

export const SHARPENING: SectionDef = {
  id: 'sharpening',
  title: 'Sharpening',
  rows: [[captureCornerBoost], [sharpenAmount, sharpenRadius, sharpenDetail, sharpenMasking]],
  clear: (edits) => {
    edits.detail.capture_sharpen = NEUTRAL_DETAIL.capture_sharpen;
  }
};

export const NOISE_REDUCTION: SectionDef = {
  id: 'noise_reduction',
  title: 'Noise Reduction',
  rows: [
    [aiDenoiseStrength, aiDenoiseDetail],
    [lumaNr, lumaNrDetail, lumaNrContrast],
    [colorNr, colorNrDetail, colorNrSmoothness]
  ],
  clear: (edits) => {
    edits.detail.ai_denoise = NEUTRAL_DETAIL.ai_denoise;
  }
};
