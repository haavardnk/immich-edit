<script lang="ts">
  import { fileStem } from '#lib/browse/bulk/bulkRules.ts';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import LoupePaneControls from '#lib/components/browse/LoupePaneControls.svelte';
  import { compare } from '#lib/stores/compare.svelte.ts';

  let { fileName }: { fileName: string } = $props();

  const position = $derived(compare.focusIndex + 1);
  const name = $derived(fileStem(fileName));
</script>

<div
  role="toolbar"
  aria-label="Pane actions"
  class="flex h-14 shrink-0 items-center gap-1 border-t border-hairline bg-editor-chrome px-2"
>
  <span
    class="grid size-7 shrink-0 place-items-center rounded-full bg-primary text-[13px] font-semibold text-light"
    aria-label="Pane {position}"
    role="img"
  >
    {position}
  </span>
  <TruncationTooltip text={fileName}>
    {#snippet child({ props })}
      <span {...props} class="min-w-0 truncate px-1 text-[13px] text-light-950/65">{name}</span>
    {/snippet}
  </TruncationTooltip>
  <div class="ml-auto flex shrink-0 items-center gap-1">
    <LoupePaneControls variant="pill" />
  </div>
</div>
