<script lang="ts">
  import ResizeHandle from '#lib/components/shell/ResizeHandle.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { MAX_MAP_PANEL, MIN_MAP_PANEL, mapPanel } from '#lib/map/mapPanel.svelte.ts';
  import type { Snippet } from 'svelte';

  let { label, shown, children }: { label: string; shown: boolean; children: Snippet } = $props();

  let resizing = $state(false);
</script>

<div
  class="relative flex shrink-0 overflow-hidden {resizing
    ? ''
    : 'motion-safe:transition-[width] motion-safe:duration-base motion-safe:ease-out'}"
  style:width="{shown ? mapPanel.width : 0}px"
>
  <aside
    aria-label={label}
    aria-hidden={!shown || undefined}
    inert={!shown}
    class="relative flex h-full shrink-0 flex-col border-s border-hairline bg-light"
    style:width="{mapPanel.width}px"
  >
    {#if layout.mode === 'desktop'}
      <ResizeHandle
        label="Resize photos panel"
        orientation="horizontal"
        value={mapPanel.width}
        min={MIN_MAP_PANEL}
        max={MAX_MAP_PANEL}
        step={16}
        shiftStep={32}
        class="absolute inset-y-0 left-0 z-40 w-3 cursor-col-resize bg-transparent outline-none after:absolute after:inset-y-0 after:left-0 after:w-px after:transition-colors hover:after:bg-primary focus-visible:after:bg-primary"
        activeClass="after:bg-primary"
        onLive={(width) => {
          resizing = true;
          mapPanel.resize(width);
        }}
        onCommit={() => {
          resizing = false;
          mapPanel.persist();
        }}
      />
    {/if}
    {@render children()}
  </aside>
</div>
