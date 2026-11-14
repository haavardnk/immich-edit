import { formatLabel, spaceLabel } from '#lib/panels/export/settings.ts';
import { pixelSize } from '#lib/panels/export/summary.ts';
import type { ExportedFile } from '#lib/stores/editor/export.svelte.ts';

const SEPARATOR = ' · ';

export function fileBytes(bytes: number): string {
  if (bytes < 1000) return `${bytes} B`;
  if (bytes < 1_000_000) return `${Math.round(bytes / 1000)} KB`;
  return `${(bytes / 1_000_000).toFixed(1)} MB`;
}

export function fileMeasure(file: Pick<ExportedFile, 'blob' | 'size'>): string {
  const bytes = fileBytes(file.blob.size);
  return file.size ? `${bytes}${SEPARATOR}${pixelSize(file.size)}` : bytes;
}

export function fileSpec(file: Pick<ExportedFile, 'options' | 'renderedAt'>): string {
  const time = file.renderedAt.toLocaleTimeString(undefined, {
    hour: '2-digit',
    minute: '2-digit'
  });
  return [
    formatLabel(file.options.format),
    spaceLabel(file.options.colorSpace),
    `rendered ${time}`
  ].join(SEPARATOR);
}
