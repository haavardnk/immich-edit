<script lang="ts">
  import { trackFraction, type DateScrub } from '#lib/browse/dateScrub.svelte.ts';
  import { monthLabel, spaced, yearMarks } from '#lib/browse/scrubberModel.ts';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { observeSize } from '#lib/actions/observeSize.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';

  let { scrub, scroller }: { scrub: DateScrub; scroller: HTMLElement | undefined } = $props();

  const HIDE_MS = 1500;
  const TRACK_GAP = 16;

  let track: HTMLDivElement | undefined = $state();
  let height = $state(0);
  let scrolling = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  const at = $derived(scrub.dragging ? scrub.previewAt : scrub.markAt);
  const years = $derived(spaced(yearMarks(scrub.model), height, 28));
  const month = $derived(scrub.month(scrub.preview));

  function measure(): void {
    height = track?.clientHeight ?? 0;
  }

  function hideLater(): void {
    clearTimeout(timer);
    timer = setTimeout(() => (scrolling = false), HIDE_MS);
  }

  function reveal(): void {
    scrolling = true;
    hideLater();
  }

  const fraction = (e: PointerEvent): number => trackFraction(track, e);

  const drag = pointerDrag({
    start: (e) => {
      clearTimeout(timer);
      scrolling = true;
      scrub.begin(fraction(e), e.pointerType);
      return {};
    },
    move: (e) => scrub.drag(fraction(e), e.pointerType),
    end: () => {
      scrub.end(false);
      hideLater();
    },
    cancel: () => {
      scrub.cancel();
      hideLater();
    }
  });

  $effect(() => {
    const element = scroller;
    if (!element) return;
    element.addEventListener('scroll', reveal, { passive: true });
    return () => {
      element.removeEventListener('scroll', reveal);
      clearTimeout(timer);
    };
  });
</script>

<div
  bind:this={track}
  use:observeSize={measure}
  class="pointer-events-none absolute right-0 bottom-6 z-10 w-0"
  style:top="{browseView.dayHeaderHeight + TRACK_GAP}px"
  data-testid="date-thumb-track"
>
  {#if scrub.dragging}
    {#each years as year (year.label)}
      <span
        class="absolute right-4 -translate-y-1/2 rounded-lg bg-light-50 px-1.5 py-px text-[10.5px] text-light-950 ring-1 ring-hairline"
        style:top="{year.at * 100}%"
        aria-hidden="true"
      >
        {year.label}
      </span>
    {/each}
    {#if month && at !== null}
      <span
        data-testid="date-thumb-bubble"
        class="absolute right-6 -translate-y-1/2 rounded-full bg-light-50 px-3.5 py-2 text-sm font-semibold whitespace-nowrap text-light-950 shadow-popover ring-1 ring-hairline"
        style:top="{at * 100}%"
        aria-hidden="true"
      >
        {monthLabel(month.month)}
      </span>
    {/if}
  {/if}
  <div
    {...scrub.slider}
    data-testid="date-thumb"
    class="absolute right-0 flex h-11 w-11 -translate-y-1/2 touch-none items-center justify-end pe-1.25 outline-none transition-opacity duration-200 select-none focus-visible:opacity-100 {scrolling ||
    scrub.dragging
      ? 'pointer-events-auto opacity-100'
      : 'opacity-0'}"
    style:top="{(at ?? 0) * 100}%"
    {...drag.on()}
    onfocus={reveal}
  >
    <span class="h-11 w-2 rounded-full bg-light-950 opacity-90 shadow-md"></span>
  </div>
</div>
