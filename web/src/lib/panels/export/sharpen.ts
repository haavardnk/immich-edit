export type SharpenMedia = 'screen' | 'matte' | 'glossy';
export type SharpenAmount = 'low' | 'standard' | 'high';

export interface ExportSharpen {
  media: SharpenMedia;
  amount: SharpenAmount;
  ppi: number;
}

export const SHARPEN_PPI = { min: 72, max: 1200, default: 300 };

export function isPrintMedia(media: SharpenMedia | 'none'): boolean {
  return media === 'matte' || media === 'glossy';
}

export function sharpenError(sharpen: ExportSharpen | null): string | null {
  if (!sharpen || !isPrintMedia(sharpen.media)) return null;
  const { ppi } = sharpen;
  if (!Number.isInteger(ppi) || ppi < SHARPEN_PPI.min || ppi > SHARPEN_PPI.max) {
    return `Enter a whole number from ${SHARPEN_PPI.min} to ${SHARPEN_PPI.max}`;
  }
  return null;
}
