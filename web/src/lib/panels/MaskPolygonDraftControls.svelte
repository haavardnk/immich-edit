<script lang="ts">
  import { layout } from '#lib/device/layout.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { MAX_POLYGON_POINTS } from '#lib/types/masks.ts';
  import { Button } from '@immich/ui';
  import { mdiCheck, mdiClose, mdiUndo } from '@mdi/js';

  let { done = true }: { done?: boolean } = $props();

  const corners = $derived(editor.polygonDraft?.points.length ?? 0);
</script>

<div class="mt-1 flex flex-col gap-1" data-testid="polygon-draft-controls">
  <div class="flex items-center gap-1">
    <span class="me-auto text-[10px] text-dark/65 tabular-nums touch:text-[12px]">
      {corners} of {MAX_POLYGON_POINTS} corners
    </span>
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      class="touch:h-10 touch:px-3 touch:text-[13px]"
      leadingIcon={mdiUndo}
      disabled={corners === 0}
      onclick={editor.undoPolygonPoint}
    >
      Undo point
    </Button>
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      class="touch:h-10 touch:px-3 touch:text-[13px]"
      leadingIcon={mdiClose}
      onclick={editor.cancelPolygon}
    >
      Cancel
    </Button>
    {#if done}
      <Button
        type="button"
        size="tiny"
        color="primary"
        class="touch:h-10 touch:px-3 touch:text-[13px]"
        leadingIcon={mdiCheck}
        disabled={corners < 3}
        onclick={() => void editor.finishPolygon()}
      >
        Done
      </Button>
    {/if}
  </div>
  <p class="text-[10px] text-dark/65 touch:text-[12px]">
    {corners < 3
      ? 'Place at least three corners on the photo.'
      : `${layout.mode === 'touch' ? 'Tap' : 'Click'} the first corner or Done to close the polygon.`}
  </p>
</div>
