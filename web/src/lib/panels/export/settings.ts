import { library } from '$lib/stores/library.svelte';
import { listAlbums } from '$lib/api/albums';
import { listTags } from '$lib/api/tags';
import { toasts } from '$lib/stores/toasts.svelte';
import { DEFAULT_FILENAME_TEMPLATE } from '$lib/filenameTemplate';
import {
  DEFAULT_RESIZE_BOX,
  DEFAULT_RESIZE_MEGAPIXELS,
  DEFAULT_RESIZE_PERCENT,
  EXPORT_MAX_EDGE,
  type ExportResize,
  type ResizeMode
} from './resize';
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
  filenameTemplate: string;
  resizeMode: ResizeMode | 'none';
  resizeWidth: number | null;
  resizeHeight: number | null;
  resizeMegapixels: number;
  resizePercent: number;
  resizeEnlarge: boolean;
}

interface Option<T extends string> {
  value: T;
  label: string;
}

export const FORMATS: Option<ExportFormat>[] = [
  { value: 'jpeg', label: 'JPEG' },
  { value: 'png', label: 'PNG' },
  { value: 'webp', label: 'WebP' },
  { value: 'avif', label: 'AVIF' },
  { value: 'heic', label: 'HEIC' },
  { value: 'tiff', label: 'TIFF' },
  { value: 'jxl', label: 'JPEG XL' }
];

export const COLOR_SPACES: Option<ColorSpaceOpt>[] = [
  { value: 'srgb', label: 'sRGB' },
  { value: 'displayp3', label: 'Display P3' }
];

export const BIT_DEPTHS: Option<BitDepthOpt>[] = [
  { value: '8', label: '8-bit' },
  { value: '16', label: '16-bit' }
];

export const PNG_COMPRESSIONS: Option<PngCompressionOpt>[] = [
  { value: 'fast', label: 'Fast' },
  { value: 'default', label: 'Default' },
  { value: 'best', label: 'Best' }
];

export const TIFF_COMPRESSIONS: Option<TiffCompressionOpt>[] = [
  { value: 'none', label: 'None' },
  { value: 'lzw', label: 'LZW' },
  { value: 'deflate', label: 'Deflate' }
];

export const RESIZE_MODES: Option<ResizeMode | 'none'>[] = [
  { value: 'none', label: 'Full size' },
  { value: 'dimensions', label: 'Dimensions' },
  { value: 'megapixels', label: 'Megapixels' },
  { value: 'percent', label: 'Percentage' }
];

const STACK_PRIMARIES: StackPrimary[] = ['edited', 'original'];

function pickOption<T extends string>(value: unknown, options: Option<T>[], fallback: T): T {
  return options.find((o) => o.value === value)?.value ?? fallback;
}

function pickBoolean(value: unknown, fallback: boolean): boolean {
  return typeof value === 'boolean' ? value : fallback;
}

function pickIds(value: unknown): string[] {
  return Array.isArray(value) ? value.filter((id): id is string => typeof id === 'string') : [];
}

function pickTemplate(template: unknown, legacySuffix: unknown): string {
  if (typeof template === 'string') return template;
  if (typeof legacySuffix === 'string' && legacySuffix.trim())
    return `{name}${legacySuffix.trim()}`;
  return DEFAULT_FILENAME_TEMPLATE;
}

function pickEdge(value: unknown, fallback: number | null): number | null {
  if (value === null) return null;
  return typeof value === 'number' &&
    Number.isInteger(value) &&
    value >= 1 &&
    value <= EXPORT_MAX_EDGE
    ? value
    : fallback;
}

function pickPositive(value: unknown, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) && value > 0 ? value : fallback;
}

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
    filenameTemplate: DEFAULT_FILENAME_TEMPLATE,
    resizeMode: 'none',
    resizeWidth: DEFAULT_RESIZE_BOX,
    resizeHeight: DEFAULT_RESIZE_BOX,
    resizeMegapixels: DEFAULT_RESIZE_MEGAPIXELS,
    resizePercent: DEFAULT_RESIZE_PERCENT,
    resizeEnlarge: false
  };
}

export function restoreExportForm(
  stored: (Partial<Record<keyof ExportForm, unknown>> & { filenameSuffix?: unknown }) | undefined
): ExportForm {
  const form = defaultExportForm();
  if (!stored) return form;
  const quality = stored.quality;
  return {
    format: pickOption(stored.format, FORMATS, form.format),
    quality:
      typeof quality === 'number' && Number.isFinite(quality)
        ? Math.min(100, Math.max(1, Math.round(quality)))
        : form.quality,
    includeExif: pickBoolean(stored.includeExif, form.includeExif),
    bitDepth: pickOption(stored.bitDepth, BIT_DEPTHS, form.bitDepth),
    pngCompression: pickOption(stored.pngCompression, PNG_COMPRESSIONS, form.pngCompression),
    tiffCompression: pickOption(stored.tiffCompression, TIFF_COMPRESSIONS, form.tiffCompression),
    lossless: pickBoolean(stored.lossless, form.lossless),
    colorSpace: pickOption(stored.colorSpace, COLOR_SPACES, form.colorSpace),
    albumIds: pickIds(stored.albumIds),
    tagIds: pickIds(stored.tagIds),
    favorite: pickBoolean(stored.favorite, form.favorite),
    stackWithOriginal: pickBoolean(stored.stackWithOriginal, form.stackWithOriginal),
    stackPrimary: STACK_PRIMARIES.find((p) => p === stored.stackPrimary) ?? form.stackPrimary,
    filenameTemplate: pickTemplate(stored.filenameTemplate, stored.filenameSuffix),
    resizeMode: pickOption(stored.resizeMode, RESIZE_MODES, form.resizeMode),
    resizeWidth: pickEdge(stored.resizeWidth, form.resizeWidth),
    resizeHeight: pickEdge(stored.resizeHeight, form.resizeHeight),
    resizeMegapixels: pickPositive(stored.resizeMegapixels, form.resizeMegapixels),
    resizePercent: pickPositive(stored.resizePercent, form.resizePercent),
    resizeEnlarge: pickBoolean(stored.resizeEnlarge, form.resizeEnlarge)
  };
}

export function formatLabel(format: ExportFormat): string {
  return FORMATS.find((f) => f.value === format)?.label ?? format;
}

export function formResize(f: ExportForm): ExportResize | null {
  if (f.resizeMode === 'none') return null;
  if (f.resizeMode === 'percent') return { mode: 'percent', percent: f.resizePercent };
  if (f.resizeMode === 'megapixels') {
    return { mode: 'megapixels', megapixels: f.resizeMegapixels, enlarge: f.resizeEnlarge };
  }
  return {
    mode: 'dimensions',
    width: f.resizeWidth,
    height: f.resizeHeight,
    enlarge: f.resizeEnlarge
  };
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
    colorSpace: f.colorSpace,
    filenameTemplate: f.filenameTemplate,
    resize: formResize(f)
  };
}

export function immichOptions(f: ExportForm): ImmichExportOptions {
  return {
    ...baseOptions(f),
    albumIds: f.albumIds,
    tagIds: f.tagIds,
    favorite: f.favorite,
    stackWithOriginal: f.stackWithOriginal,
    stackPrimary: f.stackPrimary
  };
}

let albumsRequested = false;
let tagsRequested = false;

export function ensureLibraryLoaded(form: ExportForm): void {
  if (!albumsRequested) {
    albumsRequested = true;
    void listAlbums()
      .then((a) => {
        library.albums = a.sort((x, y) => x.albumName.localeCompare(y.albumName));
        form.albumIds = form.albumIds.filter((id) => a.some((album) => album.id === id));
      })
      .catch((e: unknown) => {
        albumsRequested = false;
        toasts.fail('albums', e);
      });
  }
  if (!tagsRequested) {
    tagsRequested = true;
    void listTags()
      .then((t) => {
        library.tags = t;
        form.tagIds = form.tagIds.filter((id) => t.some((tag) => tag.id === id));
      })
      .catch((e: unknown) => {
        tagsRequested = false;
        toasts.fail('tags', e);
      });
  }
}
