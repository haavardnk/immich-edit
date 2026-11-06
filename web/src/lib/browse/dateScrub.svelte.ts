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
  loadedMonthStart,
  monthAt,
  scrubModel,
  stepMonth,
  stepYear,
  visibleMonth,
  type ScrubMonth,
  type ScrubPoint
} from '#lib/browse/scrubberModel.ts';

export type ScrubFeed = {
  readonly hasPrevious: boolean;
  currentFilter: () => BrowseFilter;
  jumpTo: (assetId: string, options?: JumpOptions) => Promise<boolean>;
};

export const KEY_IDLE_MS = 400;
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
  #seeked = -1;
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
    this.#seeked = -1;
    this.drag(fraction, pointerType);
  }

  drag(fraction: number, pointerType = 'mouse'): void {
    const point = monthAt(this.model, fraction);
    this.preview = point;
    if (!point || point.index === this.#seeked) return;
    if (this.#seeked >= 0 && pointerType === 'touch') hapticTick();
    this.#seeked = point.index;
    this.seekLoaded(point.index);
  }

  end(keepPreview: boolean): void {
    const point = this.preview;
    this.dragging = false;
    if (!keepPreview) this.preview = null;
    if (point) void this.go(point.index);
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
    if (this.seekLoaded(target)) {
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

  async go(index: number): Promise<boolean> {
    const jump = (this.#jump += 1);
    if (this.seekLoaded(index)) return true;
    const month = this.model.months[index];
    if (!month) return false;
    const feed = this.#source();
    let anchor: string | null;
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
    if (!(await feed.jumpTo(anchor, { failure: JUMP_FAILURE })) || jump !== this.#jump) {
      return false;
    }
    gridHost.get()?.scrollTo(anchor);
    return true;
  }

  private seekLoaded(index: number): boolean {
    const month = this.model.months[index];
    if (!month) return false;
    const id = loadedMonthStart(browsing.visible, month.month, this.#source().hasPrevious);
    if (!id) return false;
    gridHost.get()?.scrollTo(id);
    return true;
  }
}
