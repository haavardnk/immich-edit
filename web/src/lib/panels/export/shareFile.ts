import type { ExportFormat } from '#lib/api/export.ts';
import { formatLabel } from '#lib/panels/export/settings.ts';
import type { ExportedFile } from '#lib/stores/editor/export.svelte.ts';

type SharedFile = Pick<ExportedFile, 'blob' | 'name' | 'options'>;

export interface ShareSupport {
  shareable: boolean;
  reason: string | null;
}

const MIME_TYPES: Record<ExportFormat, string> = {
  jpeg: 'image/jpeg',
  png: 'image/png',
  webp: 'image/webp',
  avif: 'image/avif',
  heic: 'image/heic',
  tiff: 'image/tiff',
  jxl: 'image/jxl'
};

const REFUSED: readonly ExportFormat[] = ['heic', 'jxl'];

function asFile(file: SharedFile): File {
  return new File([file.blob], file.name, {
    type: file.blob.type || MIME_TYPES[file.options.format]
  });
}

export function canShareFiles(): boolean {
  return typeof navigator !== 'undefined' && typeof navigator.canShare === 'function';
}

export function saveOnly(format: ExportFormat): boolean {
  return canShareFiles() && REFUSED.includes(format);
}

export const SAVE_ONLY_REASON = `${REFUSED.map(formatLabel).join(' and ')} can't be shared from the browser, so they offer Save only.`;

export function shareSupport(file: SharedFile): ShareSupport {
  if (!canShareFiles()) return { shareable: false, reason: null };
  if (navigator.canShare({ files: [asFile(file)] })) return { shareable: true, reason: null };
  const format = file.options.format;
  return {
    shareable: false,
    reason: REFUSED.includes(format)
      ? `This browser can't share ${formatLabel(format)} files.`
      : null
  };
}

export async function shareFile(file: SharedFile): Promise<boolean> {
  try {
    await navigator.share({ files: [asFile(file)] });
    return true;
  } catch (e) {
    if (e instanceof DOMException && e.name === 'AbortError') return false;
    throw e;
  }
}
