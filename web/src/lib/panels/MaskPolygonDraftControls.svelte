<script lang="ts">
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { Button } from '@immich/ui';
  import { mdiCheck, mdiClose, mdiUndo } from '@mdi/js';

  let { done = true }: { done?: boolean } = $props();

  const corners = $derived(editor.polygonDraft?.points.length ?? 0);
</script>

<div class="mt-1 flex flex-col gap-1" data-testid="polygon-draft-controls">
  <div class="flex items-center justify-end gap-1">
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      class="touch:h-11 touch:px-3 touch:text-label"
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
      class="touch:h-11 touch:px-3 touch:text-label"
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
        class="touch:h-11 touch:px-3 touch:text-label"
        leadingIcon={mdiCheck}
        disabled={corners < 3}
        onclick={() => void editor.finishPolygon()}
      >
        Done
      </Button>
    {/if}
  </div>
</div>
