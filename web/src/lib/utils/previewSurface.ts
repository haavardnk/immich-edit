export type PreviewSurface = HTMLImageElement | HTMLCanvasElement;

export function surfaceSize(el: PreviewSurface): { w: number; h: number } {
  if (el instanceof HTMLImageElement) return { w: el.naturalWidth, h: el.naturalHeight };
  return { w: el.width, h: el.height };
}

export type UvRect = { u: number; v: number; w: number; h: number };

export function surfacePixels(el: PreviewSurface, box: UvRect, maxEdge = 256): ImageData | null {
  const size = surfaceSize(el);
  if (size.w < 1 || size.h < 1) return null;
  const x0 = Math.min(size.w - 1, Math.max(0, Math.floor(box.u * size.w)));
  const y0 = Math.min(size.h - 1, Math.max(0, Math.floor(box.v * size.h)));
  const x1 = Math.min(size.w, Math.max(x0 + 1, Math.ceil((box.u + box.w) * size.w)));
  const y1 = Math.min(size.h, Math.max(y0 + 1, Math.ceil((box.v + box.h) * size.h)));
  const scale = Math.min(1, maxEdge / Math.max(x1 - x0, y1 - y0));
  const canvas = document.createElement('canvas');
  canvas.width = Math.max(1, Math.round((x1 - x0) * scale));
  canvas.height = Math.max(1, Math.round((y1 - y0) * scale));
  const context = canvas.getContext('2d', { willReadFrequently: true });
  if (!context) return null;
  context.drawImage(el, x0, y0, x1 - x0, y1 - y0, 0, 0, canvas.width, canvas.height);
  return context.getImageData(0, 0, canvas.width, canvas.height);
}
