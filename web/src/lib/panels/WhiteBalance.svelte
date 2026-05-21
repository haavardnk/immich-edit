<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
</script>

<div class="flex flex-col gap-2.5">
  <SliderRow
    label="Temperature"
    bind:value={editor.edits.basic.wb_temp}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
    gradient="linear-gradient(to right, #4a90d9, #b8a44c)"
  />
  <SliderRow
    label="Tint"
    bind:value={editor.edits.basic.wb_tint}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
    gradient="linear-gradient(to right, #b8508a, #6ab04c)"
  />
</div>
