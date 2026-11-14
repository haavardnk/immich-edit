import { immichPreviewUrl, type ThumbSource } from '#lib/api/assets.ts';

const PREVIEW_RADIUS = 3;
const RENDER_RADIUS = 1;

export type Direction = 1 | -1;

export type Neighbour = { asset: ThumbSource; distance: number };

export type StepFn = (id: string, direction: Direction) => ThumbSource | null;

export function neighbourWindow(current: string, step: StepFn, radius: number): Neighbour[] {
  const found: Neighbour[] = [];
  for (const direction of [1, -1] as const) {
    let asset: ThumbSource | null = { id: current };
    for (let distance = 1; distance <= radius && asset !== null; distance += 1) {
      asset = step(asset.id, direction);
      if (asset !== null) found.push({ asset, distance });
    }
  }
  return found.sort((a, b) => a.distance - b.distance);
}

export function preloadUrls(
  current: ThumbSource,
  step: StepFn,
  render: (id: string) => string
): string[] {
  const urls = neighbourWindow(current.id, step, PREVIEW_RADIUS).flatMap(({ asset, distance }) =>
    distance <= RENDER_RADIUS
      ? [immichPreviewUrl(asset), render(asset.id)]
      : [immichPreviewUrl(asset)]
  );
  return [...new Set(urls)].filter((url) => url !== immichPreviewUrl(current));
}

function loadImage(src: string): HTMLImageElement {
  const image = new Image();
  image.src = src;
  return image;
}

export class ImagePreloader {
  private readonly held = new Map<string, HTMLImageElement>();

  constructor(private readonly load: (src: string) => HTMLImageElement = loadImage) {}

  keep(urls: readonly string[]): void {
    const wanted = new Set(urls);
    for (const [src, image] of this.held) {
      if (wanted.has(src)) continue;
      image.src = '';
      this.held.delete(src);
    }
    for (const src of wanted) if (!this.held.has(src)) this.held.set(src, this.load(src));
  }
}
