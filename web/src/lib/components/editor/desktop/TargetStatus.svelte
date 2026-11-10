<script lang="ts">
  import { TARGET_TOOLS } from '#lib/editor/targetTools.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { targetAdjust } from '#lib/stores/targetAdjust.svelte.ts';
  import { Icon, IconButton } from '@immich/ui';
  import { mdiClose, mdiCrosshairsGps } from '@mdi/js';

  const def = $derived(targetAdjust.tool ? TARGET_TOOLS[targetAdjust.tool] : null);
</script>

{#if def}
  <div role="status" class="flex h-7 min-w-0 flex-1 items-center gap-2 text-[11px]">
    <Icon icon={mdiCrosshairsGps} size="14px" class="shrink-0 text-primary" aria-hidden="true" />
    <span class="shrink-0 font-medium text-light-950">Targeted adjustment: {def.label}</span>
    <span class="min-w-0 truncate text-dark/65">
      Click: select {def.noun} · Drag: adjust · {keysFor('editorEscape')}: exit
    </span>
    <IconButton
      size="tiny"
      variant="ghost"
      color="secondary"
      class="size-6 shrink-0"
      icon={mdiClose}
      title="Exit targeted adjustment"
      aria-label="Exit targeted adjustment"
      onclick={targetAdjust.exit}
    />
  </div>
{/if}
