import type { ExportFormat } from '#lib/api/export.ts';

export function canCopyImages(): boolean {
  return (
    typeof ClipboardItem !== 'undefined' &&
    typeof navigator !== 'undefined' &&
    typeof navigator.clipboard?.write === 'function'
  );
}

async function pngOf(blob: Blob, format: ExportFormat): Promise<Blob> {
  if (format === 'png') return blob;
  const bitmap = await createImageBitmap(blob);
  try {
    const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
    const context = canvas.getContext('2d');
    if (!context) throw new Error('This browser cannot draw the image');
    context.drawImage(bitmap, 0, 0);
    return await canvas.convertToBlob({ type: 'image/png' });
  } finally {
    bitmap.close();
  }
}

export function copyImage(blob: Blob, format: ExportFormat): Promise<void> {
  return navigator.clipboard.write([new ClipboardItem({ 'image/png': pngOf(blob, format) })]);
}
