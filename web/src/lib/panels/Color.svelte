<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
</script>

<div class="flex flex-col gap-2.5">
  <SliderRow
    label="Saturation"
    bind:value={editor.edits.basic.saturation}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />
</div>
