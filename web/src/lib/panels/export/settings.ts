import { library } from '$lib/stores/library.svelte';
import { listAlbums } from '$lib/api/albums';
import { listTags } from '$lib/api/tags';
import { toasts } from '$lib/stores/toasts.svelte';
import type {
  BitDepthOpt,
  ColorSpaceOpt,
  ExportFormat,
  ExportOptions,
  ImmichExportOptions,
  PngCompressionOpt,
  StackPrimary,
  TiffCompressionOpt
} from '$lib/api/export';

export type Destination = 'download' | 'immich';

export const DEFAULT_FILENAME_SUFFIX = '_edit';

export interface ExportForm {
  format: ExportFormat;
  quality: number;
  includeExif: boolean;
  bitDepth: BitDepthOpt;
  pngCompression: PngCompressionOpt;
  tiffCompression: TiffCompressionOpt;
  lossless: boolean;
  colorSpace: ColorSpaceOpt;
  albumIds: string[];
  tagIds: string[];
  favorite: boolean;
  stackWithOriginal: boolean;
  stackPrimary: StackPrimary;
  filenameSuffix: string;
}

export const FORMATS: { value: ExportFormat; label: string }[] = [
  { value: 'jpeg', label: 'JPEG' },
  { value: 'png', label: 'PNG' },
  { value: 'webp', label: 'WebP' },
  { value: 'avif', label: 'AVIF' },
  { value: 'heic', label: 'HEIC' },
  { value: 'tiff', label: 'TIFF' },
  { value: 'jxl', label: 'JPEG XL' }
];

export function defaultExportForm(): ExportForm {
  return {
    format: 'jpeg',
    quality: 90,
    includeExif: true,
    bitDepth: '8',
    pngCompression: 'default',
    tiffCompression: 'lzw',
    lossless: false,
    colorSpace: 'srgb',
    albumIds: [],
    tagIds: [],
    favorite: false,
    stackWithOriginal: false,
    stackPrimary: 'edited',
    filenameSuffix: DEFAULT_FILENAME_SUFFIX
  };
}

export function formatLabel(format: ExportFormat): string {
  return FORMATS.find((f) => f.value === format)?.label ?? format;
}

export function baseOptions(f: ExportForm): ExportOptions {
  return {
    format: f.format,
    quality: f.quality,
    includeExif: f.includeExif,
    bitDepth: f.bitDepth,
    pngCompression: f.pngCompression,
    tiffCompression: f.tiffCompression,
    lossless: f.format === 'webp' ? f.lossless || f.includeExif : f.lossless,
    colorSpace: f.colorSpace
  };
}

export function immichOptions(f: ExportForm): ImmichExportOptions {
  return {
    ...baseOptions(f),
    albumIds: f.albumIds,
    tagIds: f.tagIds,
    favorite: f.favorite,
    stackWithOriginal: f.stackWithOriginal,
    stackPrimary: f.stackPrimary,
    filenameSuffix: f.filenameSuffix
  };
}

let albumsRequested = false;
let tagsRequested = false;

export function ensureLibraryLoaded(): void {
  if (!albumsRequested) {
    albumsRequested = true;
    void listAlbums()
      .then((a) => {
        library.albums = a.sort((x, y) => x.albumName.localeCompare(y.albumName));
      })
      .catch((e: unknown) => {
        albumsRequested = false;
        toasts.push('error', `albums: ${(e as Error).message}`);
      });
  }
  if (!tagsRequested) {
    tagsRequested = true;
    void listTags()
      .then((t) => {
        library.tags = t;
      })
      .catch((e: unknown) => {
        tagsRequested = false;
        toasts.push('error', `tags: ${(e as Error).message}`);
      });
  }
}
