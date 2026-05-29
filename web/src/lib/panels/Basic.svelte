<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiRestore } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';

  function resetWb(): void {
    editor.edits.basic.wb_temp = 0;
    editor.edits.basic.wb_tint = 0;
    editor.onCommit('Reset White Balance');
  }

  function resetTone(): void {
    editor.edits.basic.exposure_ev = 0;
    editor.edits.basic.brightness = 0;
    editor.edits.basic.contrast = 0;
    editor.edits.tone.highlights = 0;
    editor.edits.tone.shadows = 0;
    editor.edits.tone.whites = 0;
    editor.edits.tone.blacks = 0;
    editor.onCommit('Reset Tone');
  }

  function resetPresence(): void {
    editor.edits.basic.texture = 0;
    editor.edits.basic.clarity = 0;
    editor.edits.basic.dehaze = 0;
    editor.edits.basic.vibrance = 0;
    editor.edits.basic.saturation = 0;
    editor.onCommit('Reset Presence');
  }
</script>

<div class="flex flex-col divide-y divide-white/5">
  <div class="flex flex-col gap-2.5 pb-3">
    <div class="flex items-center justify-between">
      <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">White Balance</div>
      <button
        type="button"
        class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
        title="Reset White Balance"
        aria-label="Reset White Balance"
        onclick={resetWb}
      >
        <Icon path={mdiRestore} size={14} />
      </button>
    </div>
    <SliderRow
      label="Temperature"
      commitAction="Temperature"
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
      commitAction="Tint"
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
    <div class="flex items-center justify-between">
      <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Tone</div>
      <button
        type="button"
        class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
        title="Reset Tone"
        aria-label="Reset Tone"
        onclick={resetTone}
      >
        <Icon path={mdiRestore} size={14} />
      </button>
    </div>
    <SliderRow
      label="Exposure"
      commitAction="Exposure"
      bind:value={editor.edits.basic.exposure_ev}
      min={-5}
      max={5}
      step={0.05}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
    />
    <SliderRow
      label="Brightness"
      commitAction="Brightness"
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
      commitAction="Contrast"
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
      commitAction="Highlights"
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
      commitAction="Shadows"
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
      commitAction="Whites"
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
      commitAction="Blacks"
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
    <div class="flex items-center justify-between">
      <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Presence</div>
      <button
        type="button"
        class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
        title="Reset Presence"
        aria-label="Reset Presence"
        onclick={resetPresence}
      >
        <Icon path={mdiRestore} size={14} />
      </button>
    </div>
    <SliderRow
      label="Texture"
      commitAction="Texture"
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
      commitAction="Clarity"
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
      commitAction="Dehaze"
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
      commitAction="Vibrance"
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
      commitAction="Saturation"
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
