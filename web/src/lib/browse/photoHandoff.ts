import { isEditorPath } from '#lib/utils/route.ts';
import { boxFrame, containBox, onScreen, type Box } from '#lib/browse/zoomGeometry.ts';
import {
  animated,
  boxOf,
  clearGhosts,
  copyPicture,
  drawn,
  loaded,
  pictureRatio,
  placeGhost,
  settle,
  timing,
  viewport,
  type Picture
} from '#lib/motion/photoGhost.ts';

export type PhotoShot = { id: string; box: Box; ratio: number; picture: Picture };

type Held = PhotoShot & { at: number };

const PREVIEW = '[data-testid="preview-image"]';
const EDITOR_PHOTO = `.editor-stage ${PREVIEW}`;
const LOUPE_PHOTO = '[data-testid="loupe-track"] img:not([data-testid="loupe-neighbour"])';
const READY_WAIT_MS = 2000;
const HELD_MS = 3000;

let held: Held | null = null;

function tilePhoto(id: string): HTMLImageElement | null {
  const image = document.querySelector(`[data-asset-id="${CSS.escape(id)}"] img`);
  return loaded(image) && onScreen(boxOf(image), viewport()) ? image : null;
}

function shotOf(id: string): PhotoShot | null {
  const editor = document.querySelector(EDITOR_PHOTO);
  const loupe = [...document.querySelectorAll(LOUPE_PHOTO)].reverse().find(loaded) ?? null;
  const fitted = drawn(editor) ? editor : loupe;
  if (fitted) {
    const ratio = pictureRatio(fitted);
    return { id, box: containBox(boxOf(fitted), ratio), ratio, picture: copyPicture(fitted) };
  }
  const tile = tilePhoto(id);
  if (!tile) return null;
  return { id, box: boxOf(tile), ratio: pictureRatio(tile), picture: copyPicture(tile) };
}

function editorAssetId(url: URL | undefined): string | null {
  return url && isEditorPath(url.pathname) ? (url.pathname.split('/')[2] ?? null) : null;
}

export function handoffId(from: URL | undefined, to: URL | undefined): string | null {
  const leaving = editorAssetId(from);
  const entering = editorAssetId(to);
  if ((leaving === null) === (entering === null)) return null;
  return leaving ?? entering;
}

function fresh(id: string): boolean {
  return held?.id === id && performance.now() - held.at < HELD_MS;
}

export function holdPhoto(id: string | null): void {
  if (id && fresh(id)) return;
  const shot = id && animated() ? shotOf(id) : null;
  held = shot ? { ...shot, at: performance.now() } : null;
}

export function takePhoto(id: string): PhotoShot | null {
  const shot = fresh(id) ? held : null;
  held = null;
  return shot;
}

function whenDrawn(stage: HTMLElement): Promise<Element | null> {
  const started = performance.now();
  return new Promise((resolve) => {
    const look = (): void => {
      const photo = stage.querySelector(PREVIEW);
      if (drawn(photo)) resolve(photo);
      else if (!stage.isConnected || performance.now() - started > READY_WAIT_MS) resolve(null);
      else requestAnimationFrame(look);
    };
    look();
  });
}

function frames(count: number): Promise<void> {
  return new Promise((resolve) => {
    const tick = (left: number): void => {
      if (left === 0) resolve();
      else requestAnimationFrame(() => tick(left - 1));
    };
    tick(count);
  });
}

function glide(ghost: HTMLElement, from: Box, to: Box, duration: string): Animation {
  return ghost.animate(
    [boxFrame(from), boxFrame(to)],
    timing(duration, '--ease-emphasized', 'forwards')
  );
}

async function land(shot: PhotoShot, stage: HTMLElement): Promise<void> {
  clearGhosts();
  const ghost = placeGhost(shot.picture, shot.box);
  await frames(2);
  const target = containBox(boxOf(stage), shot.ratio);
  const [, photo] = await Promise.all([
    glide(ghost, shot.box, target, '--transition-duration-slow').finished.catch(() => undefined),
    whenDrawn(stage)
  ]);
  const settled = photo ? boxOf(photo) : target;
  const fade = ghost.animate(
    [
      { ...boxFrame(target), opacity: 1 },
      { ...boxFrame(settled), opacity: 0 }
    ],
    timing('--transition-duration-fast', '--ease-exit', 'forwards')
  );
  settle(fade, [ghost]);
}

export function landInEditor(id: string | null, stage: HTMLElement): void {
  const shot = id ? takePhoto(id) : null;
  if (shot) void land(shot, stage);
}
