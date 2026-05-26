import type { Edits } from '$lib/types/edits';
import { isIdentity } from '$lib/types/edits';

export type ExportFormat = 'jpeg' | 'png' | 'webp' | 'avif' | 'heic' | 'tiff' | 'jxl';
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
}

export const EXTENSION_BY_FORMAT: Record<ExportFormat, string> = {
  jpeg: 'jpg',
  png: 'png',
  webp: 'webp',
  avif: 'avif',
  heic: 'heic',
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
    lossless: String(opts.lossless)
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
        lossless: opts.lossless
      })
    });
  }
  if (!resp.ok) throw new Error(`export failed: ${resp.status}`);
  return resp.blob();
}

export type StackPrimary = 'edited' | 'original';

export interface ImmichExportOptions extends ExportOptions {
  albumIds: string[];
  tagIds: string[];
  favorite: boolean;
  stackWithOriginal: boolean;
  stackPrimary: StackPrimary;
  filenameSuffix: string;
}

export interface ImmichExportResult {
  asset_id: string;
  filename: string;
  status: string;
  warnings: string[];
}

export async function uploadToImmich(
  assetId: string,
  edits: Edits,
  opts: ImmichExportOptions
): Promise<ImmichExportResult> {
  const resp = await fetch(`/api/assets/${assetId}/export/immich`, {
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
      album_ids: opts.albumIds,
      tag_ids: opts.tagIds,
      favorite: opts.favorite,
      stack_with_original: opts.stackWithOriginal,
      stack_primary: opts.stackPrimary,
      filename_suffix: opts.filenameSuffix
    })
  });
  if (!resp.ok) {
    let msg = `upload failed: ${resp.status}`;
    try {
      const body = await resp.json();
      if (typeof body?.message === 'string') msg = body.message;
    } catch {
      /* ignore */
    }
    throw new Error(msg);
  }
  return (await resp.json()) as ImmichExportResult;
}
