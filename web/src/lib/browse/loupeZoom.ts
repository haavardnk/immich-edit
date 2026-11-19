import { boxFrame, containBox, onScreen, type Box } from '#lib/browse/zoomGeometry.ts';
import { prefersReducedMotion } from 'svelte/motion';

type Shot = { id: string; box: Box; src: string; backdrop: number };
type Tile = { image: HTMLImageElement; box: Box; src: string; ratio: number };

const TRACK = '[data-testid="loupe-track"]';
const SOURCE = `${TRACK} img:not([data-testid="loupe-neighbour"])`;
const SCRIM = '[data-loupe-scrim]';
const GHOST = 'pointer-events-none fixed z-50 m-0 max-w-none object-cover select-none';
const BACKDROP = 'pointer-events-none fixed inset-0 z-50 bg-image-canvas';

let captured: Shot | null = null;
let layers: Element[] = [];

function animated(): boolean {
  return typeof Element.prototype.animate === 'function' && !prefersReducedMotion.current;
}

function token(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

function timing(
  duration: string,
  easing: string,
  fill: FillMode = 'backwards'
): KeyframeAnimationOptions {
  const value = token(duration);
  const ms = value.endsWith('ms') ? Number.parseFloat(value) : Number.parseFloat(value) * 1000;
  return { duration: Number.isFinite(ms) ? ms : 0, easing: token(easing) || 'ease', fill };
}

function boxOf(element: Element): Box {
  const { left, top, width, height } = element.getBoundingClientRect();
  return { left, top, width, height };
}

function viewport(): { width: number; height: number } {
  return { width: window.innerWidth, height: window.innerHeight };
}

function loaded(image: Element | null): image is HTMLImageElement {
  return image instanceof HTMLImageElement && image.complete && image.naturalWidth > 0;
}

function tileOf(id: string): Tile | null {
  const image = document.querySelector(`[data-asset-id="${CSS.escape(id)}"] img`);
  if (!loaded(image)) return null;
  const box = boxOf(image);
  if (!onScreen(box, viewport())) return null;
  return {
    image,
    box,
    src: image.currentSrc || image.src,
    ratio: image.naturalWidth / image.naturalHeight
  };
}

function snapshot(id: string): Shot | null {
  const image = [...document.querySelectorAll(SOURCE)].reverse().find(loaded);
  if (!image) return null;
  const box = containBox(boxOf(image), image.naturalWidth / image.naturalHeight);
  const scrim = document.querySelector(SCRIM);
  const backdrop = scrim ? Number(getComputedStyle(scrim).opacity) : 1;
  return { id, box, src: image.currentSrc || image.src, backdrop };
}

function layer<T extends HTMLElement>(element: T, className: string): T {
  element.className = className;
  element.setAttribute('aria-hidden', 'true');
  document.body.append(element);
  layers.push(element);
  return element;
}

function ghost(src: string, box: Box): HTMLImageElement {
  const image = layer(document.createElement('img'), GHOST);
  image.alt = '';
  image.draggable = false;
  image.src = src;
  Object.assign(image.style, boxFrame(box));
  return image;
}

function hide(tile: Tile, motion: KeyframeAnimationOptions): void {
  tile.image.animate([{ opacity: 0 }, { opacity: 0 }], { ...motion, fill: 'none' });
}

function clear(): void {
  for (const element of layers) element.remove();
  layers = [];
}

function settle(animation: Animation, elements: Element[]): void {
  void animation.finished
    .catch(() => undefined)
    .then(() => {
      for (const element of elements) element.remove();
      layers = layers.filter((element) => !elements.includes(element));
    });
}

export function zoomIn(id: string | null, root: HTMLElement): void {
  if (!id || !animated()) return;
  clear();
  const track = root.querySelector(TRACK);
  const tile = tileOf(id);
  if (!tile || !track) {
    root.animate(
      [{ opacity: 0 }, { opacity: 1 }],
      timing('--transition-duration-base', '--ease-standard')
    );
    return;
  }
  const motion = timing('--transition-duration-slow', '--ease-emphasized');
  const to = containBox(boxOf(track), tile.ratio);
  const image = ghost(tile.src, tile.box);
  settle(
    image.animate(
      [
        { ...boxFrame(tile.box), opacity: 1 },
        { ...boxFrame(to), opacity: 1, offset: 0.8 },
        { ...boxFrame(to), opacity: 0 }
      ],
      { ...motion, fill: 'forwards' }
    ),
    [image]
  );
  track.animate([{ opacity: 0 }, { opacity: 0, offset: 0.8 }, { opacity: 1 }], motion);
  hide(tile, motion);
  root.animate(
    [{ opacity: 0 }, { opacity: 1 }],
    timing('--transition-duration-base', '--ease-standard')
  );
}

export function captureLoupe(id: string): void {
  captured = animated() ? snapshot(id) : null;
}

export function zoomOut(id: string): void {
  const shot = captured?.id === id ? captured : null;
  captured = null;
  if (!animated()) return;
  const source = shot ?? snapshot(id);
  if (!source) return;
  clear();
  const backdrop = layer(document.createElement('div'), BACKDROP);
  backdrop.style.opacity = String(source.backdrop);
  const image = ghost(source.src, source.box);
  requestAnimationFrame(() => {
    if (!image.isConnected) return;
    const motion = timing('--transition-duration-slow', '--ease-emphasized', 'forwards');
    const tile = tileOf(id);
    settle(backdrop.animate([{ opacity: source.backdrop }, { opacity: 0 }], motion), [backdrop]);
    if (tile) hide(tile, motion);
    const frames: Keyframe[] = tile
      ? [boxFrame(source.box), boxFrame(tile.box)]
      : [
          { opacity: 1, transform: 'scale(1)' },
          { opacity: 0, transform: 'scale(0.96)' }
        ];
    settle(image.animate(frames, motion), [image]);
  });
}
