export interface DetailEdits {
  capture_sharpen: boolean;
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
}

export const NEUTRAL_DETAIL: DetailEdits = {
  capture_sharpen: true,
  sharpen_amount: null,
  sharpen_radius: 1.0,
  sharpen_detail: 25,
  sharpen_masking: 0,
  luma_nr_amount: 0,
  luma_nr_detail: 50,
  luma_nr_contrast: 0,
  color_nr_amount: 0,
  color_nr_detail: 50,
  color_nr_smoothness: 50
};

export const RAW_SHARPEN_AMOUNT = 40;

export function neutralSharpenAmount(isRaw: boolean): number {
  return isRaw ? RAW_SHARPEN_AMOUNT : 0;
}
