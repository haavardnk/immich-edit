<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
</script>

<div class="flex flex-col divide-y divide-white/5">
  <div class="flex flex-col gap-2.5 pb-3">
    <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">White Balance</div>
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
  <div class="flex flex-col gap-2.5 py-3">
    <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Tone</div>
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
      label="Contrast"
      bind:value={editor.edits.basic.contrast}
      min={-100}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <div class="border-t border-white/5"></div>
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
  <div class="flex flex-col gap-2.5 pt-3">
    <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Presence</div>
    <SliderRow
      label="Texture"
      bind:value={editor.edits.basic.texture}
      min={-100}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Clarity"
      bind:value={editor.edits.basic.clarity}
      min={-100}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Dehaze"
      bind:value={editor.edits.basic.dehaze}
      min={-100}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <div class="border-t border-white/5"></div>
    <SliderRow
      label="Vibrance"
      bind:value={editor.edits.basic.vibrance}
      min={-100}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
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
</div>
