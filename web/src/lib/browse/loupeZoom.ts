import { takePhoto } from '#lib/browse/photoHandoff.ts';
import { boxFrame, containBox, onScreen, type Box } from '#lib/browse/zoomGeometry.ts';
import {
  animated,
  boxOf,
  clearGhosts,
  ghost,
  layer,
  loaded,
  placeGhost,
  settle,
  timing,
  viewport
} from '#lib/motion/photoGhost.ts';

type Shot = { id: string; box: Box; src: string; backdrop: number };
type Tile = { image: HTMLImageElement; box: Box; src: string; ratio: number };

const TRACK = '[data-testid="loupe-track"]';
const SOURCE = `${TRACK} img:not([data-testid="loupe-neighbour"])`;
const SCRIM = '[data-loupe-scrim]';
const BACKDROP = 'pointer-events-none fixed inset-0 z-50 bg-image-canvas';

let captured: Shot | null = null;

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

function hide(tile: Tile, motion: KeyframeAnimationOptions): void {
  tile.image.animate([{ opacity: 0 }, { opacity: 0 }], { ...motion, fill: 'none' });
}

function arrive(track: Element, from: HTMLElement, start: Box, ratio: number): void {
  const motion = timing('--transition-duration-slow', '--ease-emphasized');
  const to = containBox(boxOf(track), ratio);
  settle(
    from.animate(
      [
        { ...boxFrame(start), opacity: 1 },
        { ...boxFrame(to), opacity: 1, offset: 0.8 },
        { ...boxFrame(to), opacity: 0 }
      ],
      { ...motion, fill: 'forwards' }
    ),
    [from]
  );
  track.animate([{ opacity: 0 }, { opacity: 0, offset: 0.8 }, { opacity: 1 }], motion);
}

export function zoomIn(id: string | null, root: HTMLElement): void {
  const handed = id ? takePhoto(id) : null;
  if (!id || !animated()) return;
  clearGhosts();
  const track = root.querySelector(TRACK);
  if (handed && track) {
    arrive(track, placeGhost(handed.picture, handed.box), handed.box, handed.ratio);
    return;
  }
  if (document.querySelector('.editor-stage')) return;
  const tile = tileOf(id);
  if (!tile || !track) {
    root.animate(
      [{ opacity: 0 }, { opacity: 1 }],
      timing('--transition-duration-base', '--ease-standard')
    );
    return;
  }
  arrive(track, ghost(tile.src, tile.box), tile.box, tile.ratio);
  hide(tile, timing('--transition-duration-slow', '--ease-emphasized'));
  root.animate(
    [{ opacity: 0 }, { opacity: 1 }],
    timing('--transition-duration-base', '--ease-standard')
  );
}

export function captureLoupe(id: string): void {
  captured = animated() ? snapshot(id) : null;
}

type Origin = { box: Box; backdrop: number; place: () => HTMLElement };

function originOf(id: string, loupe: Shot | null): Origin | null {
  const handed = takePhoto(id);
  if (handed) {
    return { box: handed.box, backdrop: 1, place: () => placeGhost(handed.picture, handed.box) };
  }
  const shot = loupe ?? snapshot(id);
  return shot
    ? { box: shot.box, backdrop: shot.backdrop, place: () => ghost(shot.src, shot.box) }
    : null;
}

export function zoomOut(id: string): void {
  const loupe = captured?.id === id ? captured : null;
  captured = null;
  const origin = animated() ? originOf(id, loupe) : null;
  if (!origin) return;
  clearGhosts();
  const backdrop = layer(document.createElement('div'), BACKDROP);
  backdrop.style.opacity = String(origin.backdrop);
  const image = origin.place();
  requestAnimationFrame(() => {
    if (!image.isConnected) return;
    const motion = timing('--transition-duration-slow', '--ease-emphasized', 'forwards');
    const tile = tileOf(id);
    settle(backdrop.animate([{ opacity: origin.backdrop }, { opacity: 0 }], motion), [backdrop]);
    if (tile) hide(tile, motion);
    const frames: Keyframe[] = tile
      ? [boxFrame(origin.box), boxFrame(tile.box)]
      : [
          { opacity: 1, transform: 'scale(1)' },
          { opacity: 0, transform: 'scale(0.96)' }
        ];
    settle(image.animate(frames, motion), [image]);
  });
}
