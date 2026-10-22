export interface DetailEdits {
  capture_sharpen: boolean;
  capture_corner_boost: number;
  sharpen_amount: number | null;
  sharpen_radius: number;
  sharpen_detail: number;
  sharpen_masking: number;
  luma_nr_amount: number;
  luma_nr_detail: number;
  luma_nr_contrast: number;
  color_nr_amount: number;
  color_nr_detail: number;
  color_nr_smoothness: number;
  ai_denoise: boolean;
  ai_denoise_strength: number;
  ai_denoise_detail: number;
}

export const AI_DENOISE_STRENGTH = 75;

export const NEUTRAL_DETAIL: DetailEdits = {
  capture_sharpen: true,
  capture_corner_boost: 0,
  sharpen_amount: null,
  sharpen_radius: 1.0,
  sharpen_detail: 25,
  sharpen_masking: 0,
  luma_nr_amount: 0,
  luma_nr_detail: 50,
  luma_nr_contrast: 0,
  color_nr_amount: 0,
  color_nr_detail: 50,
  color_nr_smoothness: 50,
  ai_denoise: false,
  ai_denoise_strength: AI_DENOISE_STRENGTH,
  ai_denoise_detail: 0
};

export const RAW_SHARPEN_AMOUNT = 40;

export function aiDenoiseActive(detail: DetailEdits): boolean {
  return detail.ai_denoise && detail.ai_denoise_strength > 0;
}

export function neutralSharpenAmount(isRaw: boolean): number {
  return isRaw ? RAW_SHARPEN_AMOUNT : 0;
}
