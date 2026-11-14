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

export function sizeSummary(form: ExportFormValues, crop: PixelSize | null): string {
  const size = crop
    ? pixelSize(resizedSize(crop, formResize(form)))
    : labelOf(RESIZE_MODES, form.resizeMode);
  const sharpen =
    form.sharpenMedia === 'none' ? null : `Sharpen ${labelOf(SHARPEN_MEDIA, form.sharpenMedia)}`;
  return [size, sharpen].filter((part) => part !== null).join(SEPARATOR);
}

export function watermarkSummary(form: ExportFormValues, name: string | null): string {
  if (!form.watermarkId) return 'None';
  return [
    name ?? 'Watermark',
    `${form.watermarkSize}%`,
    labelOf(WATERMARK_ANCHORS, form.watermarkAnchor)
  ].join(SEPARATOR);
}

export function nameSummary(form: ExportFormValues): string {
  return form.filenameTemplate.trim() || DEFAULT_FILENAME_TEMPLATE;
}
