<script lang="ts">
  import { gridSizeHud } from '#lib/browse/gridSizeHud.svelte.ts';
  import { GRID_SIZE_LABELS } from '#lib/browse/gridSizes.ts';
  import { GRID_SIZE_OPTIONS } from '#lib/browse/viewOptions.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';

  const size = $derived(browseView.gridSize);
  const at = $derived(gridSizeHud.at);
</script>

<div role="status" class="pointer-events-none fixed top-0 left-0 z-50">
  {#if gridSizeHud.visible}
    <div
      data-testid="grid-size-hud"
      class="absolute flex h-10.5 items-center gap-2.5 rounded-full bg-light-50/95 ps-1.5 pe-3 text-[13px] whitespace-nowrap text-light-950 shadow-popover ring-1 ring-hairline backdrop-blur-md"
      style:left={at ? `clamp(10rem, ${at.x}px, calc(100vw - 10rem))` : '50vw'}
      style:top={at ? `max(3.5rem, ${at.y - 16}px)` : '50vh'}
      style:translate={at ? '-50% -100%' : '-50% -50%'}
    >
      <span class="flex gap-0.5" aria-hidden="true">
        {#each GRID_SIZE_OPTIONS as option (option.value)}
          <span
            class="grid h-7.5 min-w-8 place-items-center rounded-lg px-1.5 text-[12.5px] font-semibold {option.value ===
            size
              ? 'bg-primary/10 text-primary'
              : 'text-dark/65'}"
          >
            {option.label}
          </span>
        {/each}
      </span>
      <span class="sr-only">Thumbnail size {GRID_SIZE_LABELS[size]},</span>
      <span class="font-semibold tabular-nums">{browseView.rowHeight} px</span>
      <kbd
        class="rounded border border-hairline px-1.5 py-0.5 font-mono text-[11px] text-dark/65 touch:hidden"
      >
        {keysFor('gridWheelZoom')}
      </kbd>
    </div>
  {/if}
</div>
