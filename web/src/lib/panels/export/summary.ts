import { DEFAULT_FILENAME_TEMPLATE } from '#lib/editor/filenameTemplate.ts';
import { resizedSize, type PixelSize } from '#lib/panels/export/resize.ts';
import {
  BIT_DEPTHS,
  bitDepthShown,
  formatLabel,
  formResize,
  METADATA_OPTIONS,
  qualityShown,
  RESIZE_MODES,
  SHARPEN_MEDIA,
  spaceLabel,
  type ExportFormValues
} from '#lib/panels/export/settings.ts';
import { WATERMARK_ANCHORS } from '#lib/panels/export/watermark.ts';

const SEPARATOR = ' · ';

function labelOf<T extends string>(options: { value: T; label: string }[], value: T): string {
  return options.find((option) => option.value === value)?.label ?? value;
}

export function pixelSize(size: PixelSize): string {
  return `${size.w} × ${size.h}`;
}

export function fileSummary(form: ExportFormValues): string {
  return [
    formatLabel(form.format),
    qualityShown(form) ? `Quality ${form.quality}` : null,
    bitDepthShown(form.format) ? labelOf(BIT_DEPTHS, form.bitDepth) : null,
    form.format === 'webp' && form.lossless ? 'Lossless' : null,
    spaceLabel(form.colorSpace),
    `Metadata ${labelOf(METADATA_OPTIONS, form.metadata)}`
  ]
    .filter((part) => part !== null)
    .join(SEPARATOR);
}

function resizeLabel(form: ExportFormValues): string {
  const resize = formResize(form);
  if (!resize) return labelOf(RESIZE_MODES, form.resizeMode);
  if (resize.mode === 'percent') return `${resize.percent}%`;
  if (resize.mode === 'megapixels') return `${resize.megapixels} MP`;
  if (resize.width !== null && resize.height !== null) return `${resize.width} × ${resize.height}`;
  if (resize.width !== null) return `${resize.width} px wide`;
  return `${resize.height ?? ''} px high`;
}

function sharpenLabel(form: ExportFormValues): string | null {
  return form.sharpenMedia === 'none'
    ? null
    : `Sharpen ${labelOf(SHARPEN_MEDIA, form.sharpenMedia)}`;
}

export function sizeSummary(form: ExportFormValues, crop: PixelSize | null): string {
  const size = crop ? pixelSize(resizedSize(crop, formResize(form))) : resizeLabel(form);
  return [size, sharpenLabel(form)].filter((part) => part !== null).join(SEPARATOR);
}

export function watermarkSummary(form: ExportFormValues, name: string | null): string {
  if (!form.watermarkId) return 'None';
  return [
    name ?? 'Watermark',
    `${form.watermarkSize}%`,
    labelOf(WATERMARK_ANCHORS, form.watermarkAnchor)
  ].join(SEPARATOR);
}

export function outputSummary(
  form: ExportFormValues,
  crop: PixelSize | null
): { size: string | null; spec: string } {
  return {
    size: crop ? `${pixelSize(resizedSize(crop, formResize(form)))} px` : null,
    spec: [
      formatLabel(form.format),
      qualityShown(form) ? String(form.quality) : null,
      bitDepthShown(form.format) ? labelOf(BIT_DEPTHS, form.bitDepth) : null,
      spaceLabel(form.colorSpace)
    ]
      .filter((part) => part !== null)
      .join(SEPARATOR)
  };
}

function specLabel(form: ExportFormValues): string {
  if (qualityShown(form)) return `${formatLabel(form.format)} ${form.quality}`;
  if (bitDepthShown(form.format)) {
    return `${formatLabel(form.format)} ${labelOf(BIT_DEPTHS, form.bitDepth)}`;
  }
  return form.format === 'webp' && form.lossless ? 'WebP Lossless' : formatLabel(form.format);
}

export function presetSummary(
  form: ExportFormValues,
  immich: { albums: readonly string[] } | null
): string {
  return [
    specLabel(form),
    resizeLabel(form),
    spaceLabel(form.colorSpace),
    sharpenLabel(form),
    form.metadata === 'all' ? null : `Metadata ${labelOf(METADATA_OPTIONS, form.metadata)}`,
    form.watermarkId ? 'Watermark' : null,
    immich ? ['to Immich', ...immich.albums].join(', ') : null
  ]
    .filter((part) => part !== null)
    .join(SEPARATOR);
}

export function nameSummary(form: ExportFormValues): string {
  return form.filenameTemplate.trim() || DEFAULT_FILENAME_TEMPLATE;
}
