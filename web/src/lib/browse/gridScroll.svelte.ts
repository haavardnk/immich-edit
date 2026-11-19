import { afterNavigate } from '$app/navigation';
import { flushSync, untrack } from 'svelte';
import { observeSize } from '#lib/actions/observeSize.ts';
import type { AssetGridBox } from '#lib/browse/assetGridLayout.ts';
import { sectionTop, type DayGridLayout } from '#lib/browse/dayLayout.ts';
import { gridPath } from '#lib/browse/gridReturn.ts';
import { gridViewport } from '#lib/browse/gridViewport.svelte.ts';
import {
  anchoredViewTop,
  prependedAnchor,
  zoomAnchor,
  type TileAnchor
} from '#lib/browse/gridZoom.ts';
import type { Point } from '#lib/gestures/point.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';

const EDGE_LOAD = 400;

export type GridScrollLayout = DayGridLayout & { totalHeight: number };

export type GridScrollSource = {
  items: () => readonly AssetSummary[];
  layout: () => GridScrollLayout;
  pad: number;
  gap: number;
  loadMore: () => (() => void) | undefined;
  loadPrevious: () => (() => void) | undefined;
  loadingMore: () => boolean;
  loadingPrevious: () => boolean;
};

function findScrollParent(el: HTMLElement): HTMLElement | null {
  let parent = el.parentElement;
  while (parent) {
    const overflow = getComputedStyle(parent).overflowY;
    if (overflow === 'auto' || overflow === 'scroll') return parent;
    parent = parent.parentElement;
  }
  return null;
}

export class GridScroll {
  width = $state(0);
  height = $state(0);
  viewTop = $state(0);
  #src: GridScrollSource;
  #root: HTMLElement | null = null;
  #parent: HTMLElement | null = null;
  #key = '';
  #pending: TileAnchor | null = null;
  #top: TileAnchor | null = null;
  #knownFirst: string | null = null;
  #revealOnReturn = false;
  #revealJump: string | null = null;
  #loupeWasOpen = false;

  constructor(src: GridScrollSource) {
    this.#src = src;
    afterNavigate(() => {
      const path = gridPath(window.location);
      if (this.#key && this.#key !== path) {
        this.#key = path;
        browseView.setLastGridPath(path);
      }
      this.#settle();
    });
    $effect(() => {
      void src.items().length;
      untrack(() => this.#sync());
    });
    $effect(() => {
      const open = browseView.loupeId !== null;
      if (this.#loupeWasOpen && !open) untrack(() => this.#revealActive());
      this.#loupeWasOpen = open;
    });
  }

  get parent(): HTMLElement | null {
    return this.#parent;
  }

  attach(root: HTMLElement): () => void {
    this.#root = root;
    const path = gridPath(window.location);
    if (browseView.lastGridPath !== path) {
      selection.clear();
      browseView.setActive(null);
    } else this.#revealOnReturn = true;
    this.#key = path;
    browseView.setLastGridPath(path);
    this.#pending = browseView.getGridAnchor(path);
    const parent = findScrollParent(root);
    this.#parent = parent;
    const resize = parent ? observeSize(parent, this.onResize) : undefined;
    parent?.addEventListener('scroll', this.#onScroll, { passive: true });
    this.#sync();
    return () => {
      resize?.destroy?.();
      parent?.removeEventListener('scroll', this.#onScroll);
      gridViewport.setTop(null);
      this.#root = null;
      this.#parent = null;
    };
  }

  measure(): void {
    const root = this.#root;
    const parent = this.#parent;
    if (!root || !parent) return;
    this.width = root.clientWidth;
    this.height = parent.clientHeight;
    this.viewTop = parent.getBoundingClientRect().top - root.getBoundingClientRect().top;
    untrack(() => this.#noteTop());
  }

  onResize = (): void => {
    this.measure();
    this.#settle();
  };

  scrollToAsset = (id: string): void => {
    flushSync();
    const parent = this.#parent;
    if (!parent) return;
    const items = this.#src.items();
    const top = sectionTop(
      this.#src.layout(),
      items.findIndex((asset) => asset.id === id)
    );
    if (top === null) return;
    this.measure();
    this.#pending = null;
    parent.scrollTop += this.#src.pad + top - this.viewTop;
    this.measure();
  };

  ensureVisible = (id: string): void => {
    const parent = this.#parent;
    const index = this.#src.items().findIndex((asset) => asset.id === id);
    const box = this.#src.layout().boxes[index];
    if (!parent || !box) return;
    const rowTop = this.#src.pad + box.top;
    const rowBottom = rowTop + box.height;
    if (rowTop < this.viewTop) {
      parent.scrollTop -= this.viewTop - rowTop + this.#src.gap;
    } else if (rowBottom > this.viewTop + this.height) {
      parent.scrollTop += rowBottom - (this.viewTop + this.height) + this.#src.gap;
    }
  };

  holdAnchor = (at: Point | null): (() => void) => {
    const root = this.#root;
    if (!root || !this.#parent) return () => {};
    this.measure();
    const origin = root.getBoundingClientRect();
    const pad = this.#src.pad;
    const anchor = zoomAnchor(this.#src.layout(), {
      point: this.#inScroller(at)
        ? { x: at.x - origin.left - pad, y: at.y - origin.top - pad }
        : null,
      activeIndex: this.#src.items().findIndex((asset) => asset.id === browseView.activeId),
      viewTop: this.viewTop - pad,
      viewHeight: this.height
    });
    if (!anchor) return () => {};
    this.#pending = null;
    return () => {
      flushSync();
      this.measure();
      const top = anchoredViewTop(this.#src.layout(), anchor);
      if (top === null || !this.#parent) return;
      this.#parent.scrollTop += top - (this.viewTop - pad);
      this.measure();
    };
  };

  #inScroller(at: Point | null): at is Point {
    if (!at || !this.#parent) return false;
    const box = this.#parent.getBoundingClientRect();
    return at.y >= box.top && at.y <= box.bottom && at.x >= box.left && at.x <= box.right;
  }

  #noteTop(): void {
    const anchor = zoomAnchor(this.#src.layout(), {
      point: null,
      activeIndex: -1,
      viewTop: this.viewTop - this.#src.pad,
      viewHeight: this.height
    });
    const id = anchor ? this.#src.items()[anchor.index]?.id : undefined;
    this.#top = anchor && id ? { ...anchor, id } : null;
    gridViewport.setTop(id ?? null);
  }

  #sync(): void {
    this.#keepTopOnPrepend();
    this.measure();
    this.#settle();
  }

  #settle(): void {
    if (!this.#laidOut()) return;
    this.#restore();
    if (this.#revealOnReturn && this.#pending === null && this.#src.items().length > 0) {
      this.#revealOnReturn = !this.#revealActive();
    }
  }

  #laidOut(): boolean {
    const root = this.#root;
    if (!root || !this.#parent) return false;
    return Math.abs(root.getBoundingClientRect().height - this.#src.layout().totalHeight) < 1;
  }

  #keepTopOnPrepend(): void {
    const ids = this.#src.items().map((asset) => asset.id);
    const previous = this.#knownFirst;
    this.#knownFirst = ids[0] ?? null;
    const anchor = prependedAnchor(ids, previous, this.#top);
    const parent = this.#parent;
    if (!anchor || !parent) return;
    this.measure();
    const top = anchoredViewTop(this.#src.layout(), anchor);
    if (top === null) return;
    this.#pending = null;
    parent.scrollTop += top - (this.viewTop - this.#src.pad);
    this.measure();
  }

  #restore(): void {
    const anchor = this.#pending;
    const root = this.#root;
    const parent = this.#parent;
    if (!anchor || !root || !parent) return;
    const items = this.#src.items();
    if (items.length === 0) return;
    const index = items.findIndex((asset) => asset.id === anchor.id);
    if (index < 0) {
      this.#pending = null;
      return;
    }
    const layout = this.#src.layout();
    this.#pending = null;
    const top = anchoredViewTop(layout, { ...anchor, index });
    if (top === null) return;
    this.measure();
    parent.scrollTop += top - (this.viewTop - this.#src.pad);
    this.measure();
  }

  #save(): void {
    const parent = this.#parent;
    if (!parent || !this.#key || this.#pending !== null) return;
    browseView.setGridAnchor(this.#key, parent.scrollTop > 0 ? this.#top : null);
  }

  #onScroll = (): void => {
    this.measure();
    this.#save();
    const layout = this.#src.layout();
    const more = this.#src.loadMore();
    if (
      more &&
      !this.#src.loadingMore() &&
      layout.totalHeight - (this.viewTop + this.height) < EDGE_LOAD
    ) {
      more();
    }
    const previous = this.#src.loadPrevious();
    if (previous && !this.#src.loadingPrevious() && this.viewTop - this.#src.pad < EDGE_LOAD) {
      previous();
    }
  };

  #centerIfHidden(box: AssetGridBox): void {
    const parent = this.#parent;
    if (!parent) return;
    const rowTop = this.#src.pad + box.top;
    if (rowTop + box.height > this.viewTop && rowTop < this.viewTop + this.height) return;
    const center = rowTop + box.height / 2 - this.height / 2;
    parent.scrollTop += Math.max(0, center) - this.viewTop;
    this.measure();
  }

  #revealActive(): boolean {
    const id = browseView.activeId;
    if (!id) return true;
    const index = this.#src.items().findIndex((asset) => asset.id === id);
    const box = this.#src.layout().boxes[index];
    if (box) {
      this.#revealJump = null;
      this.#centerIfHidden(box);
      return true;
    }
    if (this.#revealJump === id) return true;
    this.#revealJump = id;
    void browsing.requestReveal(id).then((found) => {
      if (!found && browseView.activeId === id) browseView.setActive(null);
    });
    return false;
  }
}
