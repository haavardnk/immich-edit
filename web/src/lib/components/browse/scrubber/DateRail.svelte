<script lang="ts">
  import type { DateScrub } from '#lib/browse/dateScrub.svelte.ts';
  import { monthCountLabel, monthLabel, railMarks } from '#lib/browse/scrubberModel.ts';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { observeSize } from '#lib/actions/observeSize.ts';

  let { scrub }: { scrub: DateScrub } = $props();

  const TOP = 26;
  const BOTTOM = 12;

  let track: HTMLDivElement | undefined = $state();
  let height = $state(0);

  const marks = $derived(railMarks(scrub.model, height, 24, 10));
  const current = $derived(scrub.model.months[scrub.index]);
  const hovered = $derived(scrub.month(scrub.preview));

  function measure(): void {
    height = track?.clientHeight ?? 0;
  }

  function fraction(e: PointerEvent): number {
    const box = track?.getBoundingClientRect();
    return box && box.height > 0 ? (e.clientY - box.top) / box.height : 0;
  }

  const drag = pointerDrag({
    start: (e) => {
      if (e.button !== 0) return null;
      scrub.begin(fraction(e), e.pointerType);
      return {};
    },
    move: (e) => scrub.drag(fraction(e), e.pointerType),
    end: () => scrub.end(true),
    cancel: () => scrub.cancel(),
    hover: (e) => scrub.hover(fraction(e))
  });

  function onKey(e: KeyboardEvent): void {
    if (!scrub.key(e.key)) return;
    e.preventDefault();
    e.stopPropagation();
  }
</script>

<div
  role="slider"
  tabindex="0"
  aria-label="Timeline"
  aria-orientation="vertical"
  aria-valuemin={0}
  aria-valuemax={Math.max(0, scrub.model.months.length - 1)}
  aria-valuenow={scrub.index}
  aria-valuetext={current ? monthLabel(current.month) : undefined}
  data-testid="date-rail"
  class="relative z-10 w-13 shrink-0 cursor-pointer touch-none select-none outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary"
  {...drag.on()}
  onpointerleave={() => scrub.leave()}
  onkeydown={onKey}
  onkeyup={() => scrub.keyUp()}
  onblur={() => scrub.release()}
>
  <div
    bind:this={track}
    use:observeSize={measure}
    class="absolute inset-x-0"
    style:top="{TOP}px"
    style:bottom="{BOTTOM}px"
    data-testid="date-rail-track"
    aria-hidden="true"
  >
    {#each marks.years as year (year.label)}
      <span
        class="absolute right-2.5 -translate-y-full pb-1 text-[11px] leading-none font-semibold text-dark/65 tabular-nums"
        style:top="{year.at * 100}%"
      >
        {year.label}
      </span>
    {/each}
    {#each marks.dots as at (at)}
      <span
        class="absolute right-3.5 -mt-0.5 size-1.25 rounded-full bg-light-300"
        style:top="{at * 100}%"
      ></span>
    {/each}
    {#if scrub.markAt !== null}
      <span
        data-testid="date-rail-mark"
        class="absolute right-1.5 h-0.75 w-6.5 -translate-y-1/2 rounded-full bg-primary"
        style:top="{scrub.markAt * 100}%"
      ></span>
    {/if}
    {#if scrub.previewAt !== null && hovered}
      <span
        class="absolute right-0 h-0.5 w-13 -translate-y-1/2 bg-light-950"
        style:top="{scrub.previewAt * 100}%"
      ></span>
      <span
        data-testid="date-rail-bubble"
        class="pointer-events-none absolute right-14.5 -translate-y-1/2 rounded-lg bg-light-50 px-3 py-1.5 text-[13px] font-medium whitespace-nowrap text-light-950 shadow-popover ring-1 ring-hairline"
        style:top="{scrub.previewAt * 100}%"
      >
        {monthLabel(hovered.month)}<small class="ms-1.5 text-xs font-normal text-dark/65"
          >{monthCountLabel(hovered)}</small
        >
      </span>
    {/if}
  </div>
</div>
