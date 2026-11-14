import type { AssetGridLayout } from '#lib/browse/assetGridLayout.ts';
import { edgeScroller } from '#lib/browse/edgeScroll.ts';
import { tileIndexAt } from '#lib/browse/gridHit.ts';
import { assetIndexAt } from '#lib/browse/gridZoom.ts';
import { clickGuard } from '#lib/gestures/clickGuard.ts';
import { longPressRecognizer } from '#lib/gestures/longPress.ts';
import type { Point } from '#lib/gestures/point.ts';
import type { Recognizer } from '#lib/gestures/recognizer.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { hapticTick } from '#lib/utils/haptics.ts';

export type TouchRangeSource = {
  items: () => readonly AssetSummary[];
  layout: () => AssetGridLayout;
  pad: number;
  root: () => HTMLElement | null;
  scroller: () => HTMLElement | null;
};

export class TouchRange {
  target = $state<string | null>(null);
  readonly preview = $derived.by(() => {
    const target = this.target;
    if (!target) return null;
    return selection.rangeTarget(this.#ids(), target);
  });
  readonly added = $derived(this.preview ? this.preview.size - selection.count : 0);
  #src: TouchRangeSource;
  #client: Point | null = null;
  #origin: string | null = null;
  #guard = clickGuard();
  #edge = edgeScroller(
    () => this.#src.scroller(),
    () => this.#track()
  );

  readonly guard = this.#guard.attachment;

  constructor(src: TouchRangeSource) {
    this.#src = src;
  }

  get dragging(): boolean {
    return this.#client !== null;
  }

  recognizer(enabled: () => boolean): Recognizer {
    return longPressRecognizer({
      enabled,
      accepts: (point) => this.#tileAt(point) !== null,
      start: (point) => this.#start(point),
      move: (point) => this.#move(point),
      end: () => this.#end(),
      cancel: this.cancel
    });
  }

  cancel = (): void => {
    this.#client = null;
    this.#origin = null;
    this.target = null;
    this.#edge.stop();
  };

  #tileAt(point: Point): string | null {
    return this.#src.items()[this.#indexAt(point, tileIndexAt)]?.id ?? null;
  }

  #start(point: Point): void {
    const id = this.#tileAt(point);
    if (!id) return;
    this.#guard.arm();
    hapticTick();
    selection.enterMode();
    if (selection.has(id)) selection.anchorAt(id);
    else selection.toggle(id);
    browseView.setActive(id);
    this.#origin = id;
    this.#client = point;
  }

  #move(point: Point): void {
    if (!this.#client) return;
    this.#client = point;
    this.#edge.track(point.y);
    this.#track();
  }

  #end(): void {
    const target = this.target;
    if (target && target !== this.#origin) selection.range(this.#ids(), target);
    this.cancel();
  }

  #track(): void {
    const client = this.#client;
    if (!client) return;
    const id = this.#src.items()[this.#indexAt(client, assetIndexAt)]?.id ?? null;
    if (id && id !== this.target) this.target = id;
  }

  #indexAt(client: Point, find: (layout: AssetGridLayout, point: Point) => number): number {
    const root = this.#src.root();
    if (!root) return -1;
    const box = root.getBoundingClientRect();
    const pad = this.#src.pad;
    return find(this.#src.layout(), { x: client.x - box.left - pad, y: client.y - box.top - pad });
  }

  #ids(): string[] {
    return this.#src.items().map((asset) => asset.id);
  }
}
