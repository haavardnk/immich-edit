import { searchFirst } from '#lib/api/search.ts';
import { gridHost } from '#lib/commands/grid/gridHost.ts';
import { gridViewport } from '#lib/browse/gridViewport.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { browseControls } from '#lib/stores/browseControls.svelte.ts';
import type { JumpOptions } from '#lib/stores/browseFeed.svelte.ts';
import { monthCounts } from '#lib/stores/monthCounts.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { hapticTick } from '#lib/utils/haptics.ts';
import type { BrowseFilter } from '#lib/types/search.ts';
import {
  anchorFilter,
  fractionOf,
  loadedPhotoAt,
  monthAt,
  photoOffset,
  scrubModel,
  stepMonth,
  stepYear,
  visibleMonth,
  type LoadedPhoto,
  type ScrubMonth,
  type ScrubPoint
} from '#lib/browse/scrubberModel.ts';

export type ScrubFeed = {
  readonly hasPrevious: boolean;
  currentFilter: () => BrowseFilter;
  jumpTo: (assetId: string, options?: JumpOptions) => Promise<boolean>;
};

export const KEY_IDLE_MS = 400;
const MAX_HOPS = 4;
const JUMP_FAILURE = 'Jump to date';

const KEY_STEPS: Readonly<Record<string, readonly ['month' | 'year' | 'edge', number]>> = {
  ArrowDown: ['month', 1],
  ArrowRight: ['month', 1],
  ArrowUp: ['month', -1],
  ArrowLeft: ['month', -1],
  PageDown: ['year', 1],
  PageUp: ['year', -1],
  Home: ['edge', -1],
  End: ['edge', 1]
};

export class DateScrub {
  preview = $state<ScrubPoint | null>(null);
  dragging = $state(false);
  #picked = $state<number | null>(null);
  readonly model = $derived(scrubModel(monthCounts.months, browseControls.sortDir));
  readonly visible = $derived.by(() => {
    const id = gridViewport.topId;
    if (!id) return null;
    const items = browsing.visible;
    return visibleMonth(
      this.model,
      items,
      items.findIndex((item) => item.id === id)
    );
  });
  #source: () => ScrubFeed;
  #seeked: { index: number; offset: number } | null = null;
  #jump = 0;
  #keyTarget: number | null = null;
  #keyTimer: ReturnType<typeof setTimeout> | undefined;

  constructor(feed: () => ScrubFeed) {
    this.#source = feed;
  }

  get markAt(): number | null {
    return this.visible ? fractionOf(this.model, this.visible) : null;
  }

  get previewAt(): number | null {
    return this.preview ? fractionOf(this.model, this.preview) : null;
  }

  get index(): number {
    return this.#picked ?? this.visible?.index ?? 0;
  }

  month(point: ScrubPoint | null): ScrubMonth | null {
    return point ? (this.model.months[point.index] ?? null) : null;
  }

  hover(fraction: number): void {
    if (!this.dragging) this.preview = monthAt(this.model, fraction);
  }

  leave(): void {
    if (!this.dragging) this.preview = null;
  }

  begin(fraction: number, pointerType = 'mouse'): void {
    this.dragging = true;
    this.#seeked = null;
    this.drag(fraction, pointerType);
  }

  drag(fraction: number, pointerType = 'mouse'): void {
    const point = monthAt(this.model, fraction);
    this.preview = point;
    if (!point) return;
    const offset = photoOffset(this.model, point);
    const last = this.#seeked;
    if (last?.index === point.index && last.offset === offset) return;
    if (last && last.index !== point.index && pointerType === 'touch') hapticTick();
    this.#seeked = { index: point.index, offset };
    this.seekLoaded(point.index, offset);
  }

  end(keepPreview: boolean): void {
    const point = this.preview;
    this.dragging = false;
    if (!keepPreview) this.preview = null;
    if (point) void this.go(point.index, photoOffset(this.model, point));
  }

  cancel(): void {
    this.dragging = false;
    this.preview = null;
  }

  key(key: string): boolean {
    const step = KEY_STEPS[key];
    if (!step || this.model.months.length === 0) return false;
    const [unit, delta] = step;
    const from = this.index;
    const target =
      unit === 'month'
        ? stepMonth(this.model, from, delta)
        : unit === 'year'
          ? stepYear(this.model, from, delta)
          : delta < 0
            ? 0
            : this.model.months.length - 1;
    this.#picked = target;
    clearTimeout(this.#keyTimer);
    if (this.seekLoaded(target, 0)) {
      this.#jump += 1;
      this.#keyTarget = null;
      return true;
    }
    this.#keyTarget = target;
    this.#keyTimer = setTimeout(() => this.keyUp(), KEY_IDLE_MS);
    return true;
  }

  keyUp(): void {
    clearTimeout(this.#keyTimer);
    const target = this.#keyTarget;
    this.#keyTarget = null;
    if (target !== null) void this.go(target);
  }

  release(): void {
    this.keyUp();
    this.#picked = null;
  }

  async go(index: number, offset = 0): Promise<boolean> {
    const jump = (this.#jump += 1);
    const loaded = this.seekLoaded(index, offset);
    if (loaded?.exact) return true;
    const month = this.model.months[index];
    if (!month) return false;
    const feed = this.#source();
    let anchor = loaded?.id ?? null;
    if (!anchor) {
      try {
        anchor = await searchFirst(
          anchorFilter(feed.currentFilter(), month.month, browseControls.sortDir),
          browseControls.sort
        );
      } catch (e) {
        if (jump === this.#jump) toasts.fail(JUMP_FAILURE, e);
        return false;
      }
      if (!anchor || jump !== this.#jump) return false;
    }
    for (let hop = 0; hop < MAX_HOPS; hop += 1) {
      if (!(await feed.jumpTo(anchor, { failure: JUMP_FAILURE })) || jump !== this.#jump) {
        return false;
      }
      const found = this.seekLoaded(index, offset);
      if (!found) gridHost.get()?.scrollTo(anchor);
      if (!found || found.exact || found.id === anchor) return true;
      anchor = found.id;
    }
    return true;
  }

  private seekLoaded(index: number, offset: number): LoadedPhoto | null {
    const month = this.model.months[index];
    if (!month) return null;
    const found = loadedPhotoAt(browsing.visible, month.month, offset, this.#source().hasPrevious);
    if (found) gridHost.get()?.scrollTo(found.id);
    return found;
  }
}
