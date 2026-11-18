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

export function copyPicture(source: Picture): Picture {
  if (source instanceof HTMLImageElement) {
    const image = document.createElement('img');
    image.alt = '';
    image.draggable = false;
    image.src = source.currentSrc || source.src;
    return image;
  }
  const canvas = document.createElement('canvas');
  canvas.width = source.width;
  canvas.height = source.height;
  canvas.getContext('2d')?.drawImage(source, 0, 0);
  return canvas;
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

export function ghost(src: string, box: Box): HTMLImageElement {
  const image = document.createElement('img');
  image.alt = '';
  image.draggable = false;
  image.src = src;
  return placeGhost(image, box);
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
