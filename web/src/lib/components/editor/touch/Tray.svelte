<script lang="ts">
  import SheetHandle from '#lib/components/sheet/SheetHandle.svelte';
  import { sheetMotion } from '#lib/components/sheet/sheetMotion.ts';
  import { nextDetent } from '#lib/components/sheet/sheetSnap.ts';
  import { clickGuard } from '#lib/gestures/clickGuard.ts';
  import { gestures } from '#lib/gestures/gestures.ts';
  import { panRecognizer } from '#lib/gestures/pan.ts';
  import { reveal } from '#lib/motion/reveal.ts';
  import { touchUi, TRAY_DETENTS } from '#lib/stores/touchUi.svelte.ts';
  import { valueEntry } from '#lib/stores/valueEntry.svelte.ts';
  import { untrack } from 'svelte';
  import TouchPanel from './TouchPanel.svelte';

  const PEEK = 73;
  const COMPACT = 350;
  const EXPANDED_SHARE = 0.62;
  const MIN_PHOTO = 200;
  const ENTRY = 260;
  const LABELS = ['Peek', 'Compact', 'Expanded'] as const;

  let { available, docked, lift }: { available: number; docked: number; lift: number } = $props();

  const guard = clickGuard();
  const ceiling = $derived(Math.max(PEEK, available - MIN_PHOTO));
  const heights = $derived.by(() => {
    const expanded = Math.max(PEEK, Math.min(docked * EXPANDED_SHARE, docked - MIN_PHOTO));
    return [PEEK, Math.min(COMPACT, expanded), expanded].map((height) =>
      Math.min(height + lift, ceiling)
    );
  });
  const current = $derived(TRAY_DETENTS.indexOf(touchUi.detent));
  const entry = $derived(valueEntry.target !== null);
  const rest = $derived.by(() => {
    if (available <= 0) return PEEK;
    return entry ? Math.min(ENTRY, ceiling) : (heights[current] ?? PEEK);
  });

  function snap(index: number): void {
    const detent = TRAY_DETENTS[index];
    if (detent) touchUi.setDetent(detent);
  }

  const motion = sheetMotion({
    heights: () => heights,
    rest: () => rest,
    available: () => available,
    dismissable: () => false,
    grab: guard.arm,
    snap,
    dismiss: () => {}
  });

  const drag = gestures({
    recognizers: [
      panRecognizer({
        lockAxis: true,
        start: motion.begin,
        move: motion.follow,
        end: motion.release,
        cancel: motion.abort
      })
    ]
  });

  function retap(): void {
    if (touchUi.detent === 'peek') touchUi.raise();
    else touchUi.setDetent('peek');
  }

  function step(): void {
    snap(nextDetent(current, TRAY_DETENTS.length));
  }

  let settled = untrack(() => touchUi.detent);

  $effect(() => {
    const height = rest;
    const detent = touchUi.detent;
    if (motion.dragging()) return;
    const moved = !entry && detent !== settled;
    settled = detent;
    if (moved) void motion.settle(height);
    else motion.rest(height);
  });
</script>

<section
  aria-label="Adjustments"
  data-detent={touchUi.detent}
  transition:reveal
  class="flex shrink-0 flex-col overflow-hidden rounded-t-2xl border-t border-hairline bg-editor-panel motion-safe:data-snapping:transition-[height] motion-safe:data-snapping:duration-base motion-safe:data-snapping:ease-standard"
  {@attach motion.panel}
>
  <div class="flex-none select-none" {@attach guard.attachment}>
    <SheetHandle
      label="Resize adjustments"
      steppable
      detentText={LABELS[current] ?? ''}
      {drag}
      onstep={step}
    />
  </div>
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="flex min-h-0 flex-1 flex-col" onclick={touchUi.raise}>
    <TouchPanel onretap={retap} />
  </div>
</section>
