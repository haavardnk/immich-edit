import { immichPreviewUrl } from '#lib/api/assets.ts';

const PREVIEW_RADIUS = 3;
const RENDER_RADIUS = 1;

export type Direction = 1 | -1;

export type Neighbour = { id: string; distance: number };

export type StepFn = (id: string, direction: Direction) => string | null;

export function neighbourWindow(current: string, step: StepFn, radius: number): Neighbour[] {
  const found: Neighbour[] = [];
  for (const direction of [1, -1] as const) {
    let id: string | null = current;
    for (let distance = 1; distance <= radius && id !== null; distance += 1) {
      id = step(id, direction);
      if (id !== null) found.push({ id, distance });
    }
  }
  return found.sort((a, b) => a.distance - b.distance);
}

export function preloadUrls(
  current: string,
  step: StepFn,
  render: (id: string) => string
): string[] {
  const urls = neighbourWindow(current, step, PREVIEW_RADIUS).flatMap(({ id, distance }) =>
    distance <= RENDER_RADIUS ? [immichPreviewUrl(id), render(id)] : [immichPreviewUrl(id)]
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
