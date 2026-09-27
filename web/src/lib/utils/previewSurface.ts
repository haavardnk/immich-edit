export type PreviewSurface = HTMLImageElement | HTMLCanvasElement;

export function surfaceSize(el: PreviewSurface): { w: number; h: number } {
  if (el instanceof HTMLImageElement) return { w: el.naturalWidth, h: el.naturalHeight };
  return { w: el.width, h: el.height };
}
