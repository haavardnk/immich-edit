import { searchLocate } from '#lib/api/search.ts';
import { gridHost } from '#lib/commands/grid/gridHost.ts';
import { gridViewport } from '#lib/browse/gridViewport.svelte.ts';
import { DayCounts } from '#lib/browse/dayCounts.svelte.ts';
import { dayAt, type PreviewDay, type ScrubDay } from '#lib/browse/scrubDays.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { browseControls } from '#lib/stores/browseControls.svelte.ts';
import type { JumpOptions } from '#lib/stores/browseFeed.svelte.ts';
import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
import { monthCounts } from '#lib/stores/monthCounts.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { hapticTick } from '#lib/utils/haptics.ts';
import type { BrowseFilter, MonthCount, SortDir } from '#lib/types/search.ts';
import {
  captureDay,
  fractionOf,
  loadedPhotoAt,
  monthAt,
  monthLabel,
  photoOffset,
  scrubModel,
  stepMonth,
  stepYear,
  visibleMonth,
  type CaptureClock,
  type LoadedPhoto,
  type MonthAnchor,
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

export function trackFraction(track: HTMLElement | undefined, e: PointerEvent): number {
  const box = track?.getBoundingClientRect();
  return box && box.height > 0 ? (e.clientY - box.top) / box.height : 0;
}

export class DateScrub {
  preview = $state<ScrubPoint | null>(null);
  dragging = $state(false);
  #picked = $state<number | null>(null);
  #located = $state.raw<(MonthAnchor & { counts: readonly MonthCount[]; dir: SortDir }) | null>(
    null
  );
  readonly model = $derived(scrubModel(monthCounts.months, browseControls.sortDir));
  readonly visible = $derived.by(() => {
    const id = gridViewport.topId;
    if (!id) return null;
    const items = browsing.visible;
    return visibleMonth(
      this.model,
      items,
      items.findIndex((item) => item.id === id),
      this.#source().hasPrevious,
      this.anchor,
      this.#clock()
    );
  });
  #source: () => ScrubFeed;
  #clock = (): CaptureClock => (immichCapabilities.structuredSearch ? 'photo' : 'utc');
  #seeked: { index: number; offset: number } | null = null;
  #jump = 0;
  #keyTarget: number | null = null;
  #keyTimer: ReturnType<typeof setTimeout> | undefined;

  #days: DayCounts;

  constructor(feed: () => ScrubFeed) {
    this.#source = feed;
    this.#days = new DayCounts(() => this.#source().currentFilter());
  }

  private get anchor(): MonthAnchor | null {
    const located = this.#located;
    const current =
      located?.counts === monthCounts.months && located.dir === browseControls.sortDir;
    return current ? located : null;
  }

  get markAt(): number | null {
    return this.visible ? fractionOf(this.model, this.visible) : null;
  }

  get previewAt(): number | null {
    return this.preview ? fractionOf(this.model, this.preview) : null;
  }

  get previewDay(): PreviewDay | null {
    const point = this.preview;
    const month = this.month(point);
    if (!point || !month) return null;
    const offset = photoOffset(this.model, point);
    const days = this.#days.get(month.month, browseControls.sortDir);
    const loaded = this.loadedDay(month.month, offset, days);
    if (loaded) return loaded;
    return days ? dayAt(days, offset) : null;
  }

  get index(): number {
    return this.#picked ?? this.visible?.index ?? 0;
  }

  get slider() {
    const current = this.model.months[this.index];
    return {
      role: 'slider',
      tabindex: 0,
      'aria-label': 'Timeline',
      'aria-orientation': 'vertical',
      'aria-valuemin': 0,
      'aria-valuemax': Math.max(0, this.model.months.length - 1),
      'aria-valuenow': this.index,
      'aria-valuetext': current ? monthLabel(current.month) : undefined,
      onkeydown: (e: KeyboardEvent) => {
        if (!this.key(e.key)) return;
        e.preventDefault();
        e.stopPropagation();
      },
      onkeyup: () => this.keyUp(),
      onblur: () => this.release()
    } as const;
  }

  month(point: ScrubPoint | null): ScrubMonth | null {
    return point ? (this.model.months[point.index] ?? null) : null;
  }

  hover(fraction: number): void {
    if (this.dragging) return;
    this.preview = monthAt(this.model, fraction);
    this.wantDays(this.preview);
  }

  leave(): void {
    if (!this.dragging) this.preview = null;
  }

  begin(fraction: number, pointerType = 'mouse'): void {
    this.#jump += 1;
    this.dragging = true;
    this.#seeked = null;
    this.drag(fraction, pointerType);
  }

  drag(fraction: number, pointerType = 'mouse'): void {
    const point = monthAt(this.model, fraction);
    this.preview = point;
    if (!point) return;
    this.wantDays(point);
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
    if (this.seekLoaded(target, 0)?.exact) {
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
    if (this.seekLoaded(index, offset)?.exact) return true;
    const month = this.model.months[index];
    if (!month) return false;
    const counts = monthCounts.months;
    const dir = browseControls.sortDir;
    const feed = this.#source();
    let anchor: string | null;
    try {
      anchor = await searchLocate(feed.currentFilter(), dir, month.month, offset);
    } catch (e) {
      if (jump === this.#jump) toasts.fail(JUMP_FAILURE, e);
      return false;
    }
    if (!anchor || jump !== this.#jump) return false;
    if (!(await feed.jumpTo(anchor, { failure: JUMP_FAILURE })) || jump !== this.#jump) {
      return false;
    }
    this.#located = { counts, dir, month: month.month, offset, id: anchor };
    gridHost.get()?.scrollTo(anchor);
    return true;
  }

  private wantDays(point: ScrubPoint | null): void {
    const month = this.month(point);
    if (month) this.#days.want(month.month);
  }

  private loadedDay(
    month: string,
    offset: number,
    days: readonly ScrubDay[] | null
  ): PreviewDay | null {
    const items = browsing.visible;
    const clock = this.#clock();
    const found = loadedPhotoAt(
      items,
      month,
      offset,
      this.#source().hasPrevious,
      this.anchor,
      clock
    );
    if (!found?.exact) return null;
    const key = captureDay(
      items.find((entry) => entry.id === found.id),
      clock
    );
    if (!key) return null;
    return { key, count: days?.find((day) => day.key === key)?.count ?? null };
  }

  private seekLoaded(index: number, offset: number): LoadedPhoto | null {
    const month = this.model.months[index];
    if (!month) return null;
    const found = loadedPhotoAt(
      browsing.visible,
      month.month,
      offset,
      this.#source().hasPrevious,
      this.anchor,
      this.#clock()
    );
    if (found) gridHost.get()?.scrollTo(found.id);
    return found;
  }
}
