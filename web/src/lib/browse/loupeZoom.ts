import { takePhoto } from '#lib/browse/photoHandoff.ts';
import { boxFrame, containBox, onScreen, type Box } from '#lib/browse/zoomGeometry.ts';
import { layout } from '#lib/device/layout.svelte.ts';
import {
  animated,
  boxOf,
  clearGhosts,
  copyPicture,
  ghost,
  layer,
  loaded,
  placeGhost,
  settle,
  timing,
  viewport,
  whenShown
} from '#lib/motion/photoGhost.ts';

type Shot = { id: string; box: Box; picture: HTMLCanvasElement; backdrop: number };
type Tile = { image: HTMLImageElement; box: Box; ratio: number };

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
    ratio: image.naturalWidth / image.naturalHeight
  };
}

function backdropNow(): number {
  const scrim = document.querySelector(SCRIM);
  return scrim ? Number(getComputedStyle(scrim).opacity) : 1;
}

function snapshot(id: string): Shot | null {
  const image = [...document.querySelectorAll(SOURCE)].reverse().find(loaded);
  if (!image) return null;
  const box = containBox(boxOf(image), image.naturalWidth / image.naturalHeight);
  return { id, box, picture: copyPicture(image), backdrop: backdropNow() };
}

function hide(tile: Tile, motion: KeyframeAnimationOptions): void {
  tile.image.animate([{ opacity: 0 }, { opacity: 0 }], { ...motion, fill: 'none' });
}

const READY_MS = 1500;

export function flightDuration(): string {
  return layout.mode === 'touch' ? '--transition-duration-slow' : '--transition-duration-base';
}

function flightTiming(fill?: FillMode): KeyframeAnimationOptions {
  return timing(flightDuration(), '--ease-emphasized', fill);
}

function arrive(track: Element, from: HTMLElement, start: Box, ratio: number): void {
  const hold = track.animate([{ opacity: 0 }, { opacity: 0 }], {
    duration: READY_MS * 2,
    fill: 'forwards'
  });
  const flight = from.animate(
    [boxFrame(start), boxFrame(containBox(boxOf(track), ratio))],
    flightTiming('forwards')
  );
  const image = [...track.querySelectorAll('img:not([data-testid="loupe-neighbour"])')].at(-1);
  void Promise.all([
    flight.finished.catch(() => undefined),
    whenShown(image instanceof HTMLImageElement ? image : null, READY_MS)
  ]).then(() => {
    hold.cancel();
    settle(
      from.animate(
        [{ opacity: 1 }, { opacity: 0 }],
        timing('--transition-duration-fast', '--ease-exit', 'forwards')
      ),
      [from]
    );
  });
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
  const tile = tileOf(id);
  if (!tile || !track) {
    root.animate(
      [{ opacity: 0 }, { opacity: 1 }],
      timing('--transition-duration-base', '--ease-standard')
    );
    return;
  }
  arrive(track, ghost(tile.image, tile.box), tile.box, tile.ratio);
  hide(tile, flightTiming());
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
    const track = document.querySelector(TRACK);
    const box = track ? containBox(boxOf(track), handed.ratio) : handed.box;
    return { box, backdrop: 1, place: () => placeGhost(handed.picture, box) };
  }
  const shot = loupe ?? snapshot(id);
  if (shot) {
    return {
      box: shot.box,
      backdrop: shot.backdrop,
      place: () => placeGhost(shot.picture, shot.box)
    };
  }
  return fromTile(id);
}

function fromTile(id: string): Origin | null {
  const tile = tileOf(id);
  const track = document.querySelector(TRACK);
  if (!tile || !track) return null;
  const box = containBox(boxOf(track), tile.ratio);
  return { box, backdrop: backdropNow(), place: () => ghost(tile.image, box) };
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
    const motion = flightTiming('forwards');
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
