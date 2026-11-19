import type { AssetGridLayout } from '#lib/browse/assetGridLayout.ts';
import { edgeScroller } from '#lib/browse/edgeScroll.ts';
import { marqueeHits, rectBetween, type GridRect } from '#lib/browse/gridHit.ts';
import { assetIndexAt } from '#lib/browse/gridZoom.ts';
import { pointerDrag } from '#lib/gestures/drag.ts';
import type { Point } from '#lib/gestures/point.ts';
import { DEFAULT_THRESHOLDS } from '#lib/gestures/thresholds.ts';
import { held } from '#lib/input/modifiers.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import type { Attachment } from 'svelte/attachments';

export type MarqueeSource = {
  items: () => readonly AssetSummary[];
  layout: () => AssetGridLayout;
  pad: number;
  enabled: () => boolean;
};

type OriginTile = { id: string; dy: number };

type Drag = {
  start: Point;
  origin: Point;
  tile: OriginTile | null;
  client: Point;
  add: boolean;
  base: ReadonlySet<string>;
  before: ReadonlySet<string>;
  anchorId: string | null;
  hits: string | null;
  live: boolean;
  done: boolean;
};

const BLOCKERS = 'a, button, input, select, textarea, [role="group"]';
const CHIP_GAP = 14;

function startsOnGap(target: EventTarget | null): boolean {
  return !(target instanceof Element) || target.closest(BLOCKERS) === null;
}

function beside(at: number, size: number, room: number): number {
  return Math.max(0, at + CHIP_GAP + size > room ? at - CHIP_GAP - size : at + CHIP_GAP);
}

function inside(element: HTMLElement, e: PointerEvent): boolean {
  const box = element.getBoundingClientRect();
  return (
    e.clientX >= box.left &&
    e.clientX <= box.right &&
    e.clientY >= box.top &&
    e.clientY <= box.bottom
  );
}

export class GridMarquee {
  active = $state(false);
  covered = $state(0);
  adding = $state(false);
  #src: MarqueeSource;
  #root: HTMLElement | null = null;
  #scroller: HTMLElement | null = null;
  #box: HTMLElement | null = null;
  #chip: HTMLElement | null = null;
  #drag: Drag | null = null;
  #frame: number | null = null;
  #edge = edgeScroller(
    () => this.#scroller,
    () => this.#paint()
  );
  #pointer = pointerDrag<Drag>({
    start: (e) => this.#start(e),
    move: (e, drag) => this.#move(e, drag),
    end: (_e, drag) => this.#end(drag),
    cancel: () => this.cancel()
  });

  constructor(src: MarqueeSource) {
    this.#src = src;
  }

  get dragging(): boolean {
    return this.#drag?.live === true;
  }

  boxRef: Attachment<HTMLElement> = (element) => {
    this.#box = element;
    this.#paint();
    return () => {
      if (this.#box === element) this.#box = null;
    };
  };

  chipRef: Attachment<HTMLElement> = (element) => {
    this.#chip = element;
    this.#paint();
    return () => {
      if (this.#chip === element) this.#chip = null;
    };
  };

  attach(root: HTMLElement, scroller: HTMLElement): () => void {
    this.#root = root;
    this.#scroller = scroller;
    const listening = new AbortController();
    const options = { signal: listening.signal };
    const follow = this.#pointer.follow;
    scroller.addEventListener('pointerdown', (e) => void this.#pointer.begin(e), options);
    scroller.addEventListener('pointermove', follow.onpointermove, options);
    scroller.addEventListener('pointerup', follow.onpointerup, options);
    scroller.addEventListener('pointercancel', follow.onpointercancel, options);
    scroller.addEventListener('lostpointercapture', follow.onlostpointercapture, options);
    return () => {
      listening.abort();
      this.#finish();
      this.#root = null;
      this.#scroller = null;
    };
  }

  cancel = (): void => {
    const drag = this.#drag;
    if (drag?.live) selection.replace(drag.before, drag.anchorId);
    this.#finish();
  };

  #start(e: PointerEvent): Drag | null {
    if (e.pointerType === 'touch' || e.button !== 0 || this.#drag) return null;
    if (!this.#src.enabled() || !startsOnGap(e.target)) return null;
    const root = this.#root;
    if (!root || !inside(root, e)) return null;
    const origin = this.#toGrid({ x: e.clientX, y: e.clientY });
    if (!origin) return null;
    const drag: Drag = {
      start: { x: e.clientX, y: e.clientY },
      origin,
      tile: this.#tileAt(origin),
      client: { x: e.clientX, y: e.clientY },
      add: held('extend', e) || held('toggle', e),
      base: new Set(),
      before: new Set(),
      anchorId: null,
      hits: null,
      live: false,
      done: false
    };
    this.#drag = drag;
    return drag;
  }

  #move(e: PointerEvent, drag: Drag): void {
    if (drag.done) return;
    drag.client = { x: e.clientX, y: e.clientY };
    if (!drag.live) {
      const travel = Math.hypot(e.clientX - drag.start.x, e.clientY - drag.start.y);
      if (travel <= DEFAULT_THRESHOLDS.clickTravel) return;
      drag.live = true;
      drag.before = new Set(selection.selected);
      drag.anchorId = selection.anchorId;
      drag.base = drag.add ? drag.before : new Set();
      this.covered = 0;
      this.adding = drag.add;
      this.active = true;
      this.#paint();
    }
    this.#edge.track(e.clientY);
    this.#schedule();
  }

  #end(drag: Drag): void {
    if (!drag.done && this.#frame !== null) this.#paint();
    this.#finish();
  }

  #schedule(): void {
    if (this.#frame !== null || !this.#drag?.live) return;
    this.#frame = requestAnimationFrame(() => this.#paint());
  }

  #paint(): void {
    if (this.#frame !== null) cancelAnimationFrame(this.#frame);
    this.#frame = null;
    const drag = this.#drag;
    if (!drag?.live || drag.done) return;
    const point = this.#toGrid(drag.client);
    if (!point) return;
    const rect = rectBetween(this.#originNow(drag), point);
    this.#select(drag, this.#hitIds(rect));
    this.#place(rect, point);
  }

  #tileAt(point: Point): OriginTile | null {
    const layout = this.#src.layout();
    const index = assetIndexAt(layout, point);
    const box = layout.boxes[index];
    const asset = this.#src.items()[index];
    return box && asset ? { id: asset.id, dy: point.y - box.top } : null;
  }

  #originNow(drag: Drag): Point {
    const tile = drag.tile;
    if (!tile) return drag.origin;
    const index = this.#src.items().findIndex((asset) => asset.id === tile.id);
    const box = this.#src.layout().boxes[index];
    return box ? { x: drag.origin.x, y: box.top + tile.dy } : drag.origin;
  }

  #hitIds(rect: GridRect): string[] {
    const items = this.#src.items();
    return marqueeHits(this.#src.layout(), rect).flatMap((index) => {
      const asset = items[index];
      return asset ? [asset.id] : [];
    });
  }

  #place(rect: GridRect, point: Point): void {
    const pad = this.#src.pad;
    if (this.#box) {
      this.#box.style.transform = `translate(${pad + rect.left}px, ${pad + rect.top}px)`;
      this.#box.style.width = `${rect.right - rect.left}px`;
      this.#box.style.height = `${rect.bottom - rect.top}px`;
    }
    const chip = this.#chip;
    const root = this.#root;
    if (!chip || !root) return;
    const x = beside(point.x, chip.offsetWidth, root.clientWidth - pad * 2);
    const y = beside(point.y, chip.offsetHeight, root.clientHeight - pad * 2);
    chip.style.transform = `translate(${pad + x}px, ${pad + y}px)`;
  }

  #select(drag: Drag, ids: readonly string[]): void {
    const key = ids.join(',');
    if (key === drag.hits) return;
    drag.hits = key;
    this.covered = ids.length;
    selection.replace([...drag.base, ...ids], ids.at(-1) ?? drag.anchorId);
  }

  #toGrid(client: Point): Point | null {
    const root = this.#root;
    if (!root) return null;
    const box = root.getBoundingClientRect();
    const pad = this.#src.pad;
    return {
      x: Math.min(Math.max(client.x - box.left - pad, -pad), box.width - pad),
      y: Math.min(Math.max(client.y - box.top - pad, -pad), box.height - pad)
    };
  }

  #finish(): void {
    const drag = this.#drag;
    if (drag) drag.done = true;
    this.#drag = null;
    this.#edge.stop();
    if (this.#frame !== null) cancelAnimationFrame(this.#frame);
    this.#frame = null;
    this.active = false;
  }
}
