<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
</script>

<div class="flex flex-col gap-2.5">
  <SliderRow
    label="Exposure"
    bind:value={editor.edits.basic.exposure_ev}
    min={-5}
    max={5}
    step={0.05}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
  />
  <SliderRow
    label="Brightness"
    bind:value={editor.edits.basic.brightness}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />
  <SliderRow
    label="Contrast"
    bind:value={editor.edits.basic.contrast}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />
  <SliderRow
    label="Highlights"
    bind:value={editor.edits.tone.highlights}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />
  <SliderRow
    label="Shadows"
    bind:value={editor.edits.tone.shadows}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />
  <SliderRow
    label="Whites"
    bind:value={editor.edits.tone.whites}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />
  <SliderRow
    label="Blacks"
    bind:value={editor.edits.tone.blacks}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />
</div>
