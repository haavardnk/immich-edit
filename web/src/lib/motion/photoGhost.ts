import { boxFrame, type Box } from '#lib/browse/zoomGeometry.ts';
import { cssToken, tokenMs } from '#lib/motion/tokens.ts';
import { prefersReducedMotion } from 'svelte/motion';

export type Picture = HTMLImageElement | HTMLCanvasElement;

const GHOST = 'pointer-events-none fixed z-50 m-0 max-w-none object-cover select-none';

let layers: Element[] = [];

export function animated(): boolean {
  return (
    typeof Element !== 'undefined' &&
    typeof Element.prototype.animate === 'function' &&
    !prefersReducedMotion.current
  );
}

export function timing(
  duration: string,
  easing: string,
  fill: FillMode = 'backwards'
): KeyframeAnimationOptions {
  return { duration: tokenMs(duration), easing: cssToken(easing) || 'ease', fill };
}

export function boxOf(element: Element): Box {
  const { left, top, width, height } = element.getBoundingClientRect();
  return { left, top, width, height };
}

export function viewport(): { width: number; height: number } {
  return { width: window.innerWidth, height: window.innerHeight };
}

export function loaded(image: Element | null): image is HTMLImageElement {
  return image instanceof HTMLImageElement && image.complete && image.naturalWidth > 0;
}

export function drawn(element: Element | null): element is Picture {
  if (element instanceof HTMLCanvasElement) return element.width > 0 && element.height > 0;
  return loaded(element);
}

export function pictureRatio(picture: Picture): number {
  return picture instanceof HTMLImageElement
    ? picture.naturalWidth / picture.naturalHeight
    : picture.width / picture.height;
}

const COPY_EDGE = 2048;

export function copyPicture(source: Picture): HTMLCanvasElement {
  const width = source instanceof HTMLImageElement ? source.naturalWidth : source.width;
  const height = source instanceof HTMLImageElement ? source.naturalHeight : source.height;
  const scale = Math.min(1, COPY_EDGE / Math.max(width, height, 1));
  const canvas = document.createElement('canvas');
  canvas.width = Math.max(1, Math.round(width * scale));
  canvas.height = Math.max(1, Math.round(height * scale));
  canvas.getContext('2d')?.drawImage(source, 0, 0, canvas.width, canvas.height);
  return canvas;
}

function decoded(image: HTMLImageElement): Promise<void> {
  return image.decode().catch(() => undefined);
}

export function whenShown(image: HTMLImageElement | null, limitMs: number): Promise<void> {
  if (!image) return Promise.resolve();
  const ready =
    image.complete && image.naturalWidth > 0
      ? decoded(image)
      : new Promise<void>((resolve) => {
          image.addEventListener('load', () => void decoded(image).then(resolve), { once: true });
          image.addEventListener('error', () => resolve(), { once: true });
        });
  return Promise.race([ready, new Promise<void>((resolve) => setTimeout(resolve, limitMs))]);
}

export function layer<T extends HTMLElement>(element: T, className: string): T {
  element.className = className;
  element.setAttribute('aria-hidden', 'true');
  document.body.append(element);
  layers.push(element);
  return element;
}

export function placeGhost<T extends HTMLElement>(picture: T, box: Box): T {
  const ghost = layer(picture, GHOST);
  Object.assign(ghost.style, boxFrame(box));
  return ghost;
}

export function ghost(source: Picture, box: Box): HTMLCanvasElement {
  return placeGhost(copyPicture(source), box);
}

export function clearGhosts(): void {
  for (const element of layers) element.remove();
  layers = [];
}

export function settle(animation: Animation, elements: Element[]): void {
  void animation.finished
    .catch(() => undefined)
    .then(() => {
      for (const element of elements) element.remove();
      layers = layers.filter((element) => !elements.includes(element));
    });
}
