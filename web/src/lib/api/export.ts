import type { Edits } from '$lib/types/edits';
import { isIdentity } from '$lib/types/edits';

export type ExportFormat = 'jpeg' | 'png' | 'webp' | 'avif' | 'tiff' | 'jxl';
export type BitDepthOpt = '8' | '16';
export type PngCompressionOpt = 'fast' | 'default' | 'best';
export type TiffCompressionOpt = 'none' | 'lzw' | 'deflate';

export interface ExportOptions {
  format: ExportFormat;
  quality: number;
  includeExif: boolean;
  bitDepth: BitDepthOpt;
  pngCompression: PngCompressionOpt;
  tiffCompression: TiffCompressionOpt;
  lossless: boolean;
  speed: number;
}

export const EXTENSION_BY_FORMAT: Record<ExportFormat, string> = {
  jpeg: 'jpg',
  png: 'png',
  webp: 'webp',
  avif: 'avif',
  tiff: 'tif',
  jxl: 'jxl'
};

function paramsObject(opts: ExportOptions): Record<string, string> {
  return {
    format: opts.format,
    quality: String(opts.quality),
    include_exif: String(opts.includeExif),
    bit_depth: opts.bitDepth,
    png_compression: opts.pngCompression,
    tiff_compression: opts.tiffCompression,
    lossless: String(opts.lossless),
    speed: String(opts.speed)
  };
}

function queryString(opts: ExportOptions): string {
  return '?' + new URLSearchParams(paramsObject(opts)).toString();
}

export function exportUrlPersisted(assetId: string, opts: ExportOptions): string {
  return `/api/assets/${assetId}/export${queryString(opts)}`;
}

export async function downloadExport(
  assetId: string,
  edits: Edits,
  opts: ExportOptions
): Promise<Blob> {
  const base = `/api/assets/${assetId}/export`;
  let resp: Response;
  if (isIdentity(edits)) {
    resp = await fetch(base + queryString(opts));
  } else {
    resp = await fetch(base, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        edits,
        format: opts.format,
        quality: opts.quality,
        include_exif: opts.includeExif,
        bit_depth: opts.bitDepth,
        png_compression: opts.pngCompression,
        tiff_compression: opts.tiffCompression,
        lossless: opts.lossless,
        speed: opts.speed
      })
    });
  }
  if (!resp.ok) throw new Error(`export failed: ${resp.status}`);
  return resp.blob();
}
