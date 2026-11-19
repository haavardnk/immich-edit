<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { retouchView } from '#lib/stores/editor/retouchView.svelte.ts';
  import {
    mdiCrosshairsOff,
    mdiDeleteSweepOutline,
    mdiEyeRefreshOutline,
    mdiSelectionRemove
  } from '@mdi/js';

  let { open, onclose }: { open: boolean; onclose: () => void } = $props();
</script>

<ActionSheet
  {open}
  onOpenChange={(next) => {
    if (!next) onclose();
  }}
  title="Retouch"
  caption
>
  <MenuRow
    icon={mdiEyeRefreshOutline}
    label="Visualize spots"
    pressed={retouchView.visualize}
    onclick={retouchView.toggleVisualize}
  />
  <MenuRow
    icon={mdiCrosshairsOff}
    label="Clear source"
    disabled={!editor.retouchAnchor}
    onclick={editor.clearRetouchAnchor}
  />
  {#if editor.activeRetouchId}
    <MenuRow
      icon={mdiSelectionRemove}
      label="Deselect"
      onclick={() => (editor.activeRetouchId = null)}
    />
  {/if}
  <MenuRow
    icon={mdiDeleteSweepOutline}
    label="Remove all"
    danger
    disabled={editor.edits.retouch.length === 0}
    onclick={() => void editor.clearRetouch()}
  />
</ActionSheet>
