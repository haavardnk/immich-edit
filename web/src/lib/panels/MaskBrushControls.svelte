<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';

  const size = $derived(editor.brushTool.size);
  const hardness = $derived(editor.brushTool.hardness);
  const flow = $derived(editor.brushTool.flow);
</script>

<div class="mt-2 flex flex-col gap-2">
  <div class="flex items-center justify-between px-1">
    <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Brush</div>
    <div class="flex rounded ring-1 ring-white/10 overflow-hidden text-[10px]">
      <button
        type="button"
        class="px-2 leading-5 transition-colors {editor.brushTool.mode === 'paint'
          ? 'bg-white/15 text-immich-dark-fg'
          : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
        onclick={() => editor.setBrushTool({ mode: 'paint' })}
      >
        Paint
      </button>
      <button
        type="button"
        class="px-2 leading-5 transition-colors {editor.brushTool.mode === 'erase'
          ? 'bg-white/15 text-immich-dark-fg'
          : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
        onclick={() => editor.setBrushTool({ mode: 'erase' })}
      >
        Erase
      </button>
    </div>
  </div>
  <SliderRow
    label="Size"
    value={size}
    min={0.005}
    max={0.5}
    step={0.005}
    defaultValue={0.08}
    onLive={(v: number) => editor.setBrushTool({ size: v })}
    onCommit={() => editor.setBrushTool({ size })}
    format={(v: number) => v.toFixed(3)}
  />
  <SliderRow
    label="Hardness"
    value={hardness}
    min={0}
    max={1}
    step={0.01}
    defaultValue={0.5}
    onLive={(v: number) => editor.setBrushTool({ hardness: v })}
    onCommit={() => editor.setBrushTool({ hardness })}
    format={(v: number) => v.toFixed(2)}
  />
  <SliderRow
    label="Flow"
    value={flow}
    min={0.01}
    max={1}
    step={0.01}
    defaultValue={0.8}
    onLive={(v: number) => editor.setBrushTool({ flow: v })}
    onCommit={() => editor.setBrushTool({ flow })}
    format={(v: number) => v.toFixed(2)}
  />
</div>
