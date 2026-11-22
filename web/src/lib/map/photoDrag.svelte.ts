import { thumbUrl } from '#lib/api/assets.ts';
import type { TileDrag, TouchPlace } from '#lib/browse/tileDrag.ts';
import type { Point } from '#lib/gestures/point.ts';
import { TOUCH_REACH } from '#lib/map/markerDrag.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';

const PHOTO_DRAG_TYPE = 'application/x-immich-edit-photos';

const GHOST_OFFSET = { x: -16, y: 72 };

export interface PlaceTarget {
  leaves(point: Point): boolean;
  accepts(point: Point): boolean;
  lift(): void;
  drop(ids: readonly string[], point: Point): void;
  settle(): void;
}

function ghost(asset: AssetSummary, count: number): HTMLElement {
  const element = document.createElement('div');
  element.className =
    'fixed -top-40 -left-40 size-14 rounded-lg bg-subtle shadow-lg ring-2 ring-white';
  const image = document.createElement('img');
  image.src = thumbUrl(asset.id, 'thumbnail');
  image.alt = '';
  image.className = 'size-full rounded-lg object-cover';
  element.append(image);
  if (count > 1) {
    const badge = document.createElement('span');
    badge.className =
      'absolute -top-2 -right-2 grid h-5 min-w-5 place-items-center rounded-full bg-primary px-1 text-micro font-semibold text-light tabular-nums';
    badge.textContent = String(count);
    element.append(badge);
  }
  document.body.append(element);
  return element;
}

export class PhotoDrag implements TileDrag {
  ids = $state.raw<readonly string[] | null>(null);
  over = $state(false);
  client = $state.raw<Point | null>(null);
  readonly touch: TouchPlace;
  #target: PlaceTarget;

  constructor(target: PlaceTarget) {
    this.#target = target;
    this.touch = {
      escapes: (point) => this.#target.leaves(point),
      begin: (ids, point) => {
        this.ids = ids;
        this.#target.lift();
        this.touch.move(point);
      },
      move: (point) => {
        const tip = { x: point.x, y: point.y - TOUCH_REACH };
        this.client = tip;
        this.over = this.#target.accepts(tip);
      },
      drop: (point) => {
        const ids = this.ids;
        const tip = { x: point.x, y: point.y - TOUCH_REACH };
        if (ids && this.#target.accepts(tip)) this.#target.drop(ids, tip);
        this.end();
        this.#target.settle();
      },
      cancel: () => {
        this.end();
        this.#target.settle();
      }
    };
  }

  start(asset: AssetSummary, event: DragEvent): void {
    const ids = selection.has(asset.id) ? [...selection.selected] : [asset.id];
    this.ids = ids;
    const transfer = event.dataTransfer;
    if (!transfer) return;
    transfer.effectAllowed = 'move';
    transfer.setData(PHOTO_DRAG_TYPE, JSON.stringify(ids));
    const image = ghost(asset, ids.length);
    transfer.setDragImage(image, GHOST_OFFSET.x, GHOST_OFFSET.y);
    requestAnimationFrame(() => image.remove());
  }

  end(): void {
    this.ids = null;
    this.over = false;
    this.client = null;
  }
}
