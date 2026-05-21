<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
</script>

<div class="flex flex-col gap-2.5">
  <SliderRow
    label="Temperature"
    bind:value={editor.edits.wb_temp}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />
  <SliderRow
    label="Tint"
    bind:value={editor.edits.wb_tint}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />
</div>
