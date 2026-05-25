<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiRestore } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { NEUTRAL_DETAIL } from '$lib/types/edits';

  const sharpenInactive = $derived(editor.edits.detail.sharpen_amount === 0);
  const lumaNrInactive = $derived(editor.edits.detail.luma_nr_amount === 0);
  const colorNrInactive = $derived(editor.edits.detail.color_nr_amount === 0);

  function resetSharpen(): void {
    editor.edits.detail.sharpen_amount = NEUTRAL_DETAIL.sharpen_amount;
    editor.edits.detail.sharpen_radius = NEUTRAL_DETAIL.sharpen_radius;
    editor.edits.detail.sharpen_detail = NEUTRAL_DETAIL.sharpen_detail;
    editor.edits.detail.sharpen_masking = NEUTRAL_DETAIL.sharpen_masking;
    editor.onCommit();
  }

  function resetNr(): void {
    editor.edits.detail.luma_nr_amount = NEUTRAL_DETAIL.luma_nr_amount;
    editor.edits.detail.luma_nr_detail = NEUTRAL_DETAIL.luma_nr_detail;
    editor.edits.detail.luma_nr_contrast = NEUTRAL_DETAIL.luma_nr_contrast;
    editor.edits.detail.color_nr_amount = NEUTRAL_DETAIL.color_nr_amount;
    editor.edits.detail.color_nr_detail = NEUTRAL_DETAIL.color_nr_detail;
    editor.edits.detail.color_nr_smoothness = NEUTRAL_DETAIL.color_nr_smoothness;
    editor.onCommit();
  }
</script>

<div class="flex flex-col divide-y divide-white/5">
  <div class="flex flex-col gap-2.5 pb-3">
    <div class="flex items-center justify-between">
      <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Sharpening</div>
      <button
        type="button"
        class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
        title="Reset Sharpening"
        aria-label="Reset Sharpening"
        onclick={resetSharpen}
      >
        <Icon path={mdiRestore} size={14} />
      </button>
    </div>
    <SliderRow
      label="Amount"
      bind:value={editor.edits.detail.sharpen_amount}
      min={0}
      max={150}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Radius"
      bind:value={editor.edits.detail.sharpen_radius}
      min={0.5}
      max={3.0}
      step={0.1}
      defaultValue={1.0}
      disabled={sharpenInactive}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      onPreviewStart={() => editor.onPreview('sharpen_radius')}
      onPreviewEnd={editor.endPreview}
      format={(v: number) => v.toFixed(1)}
    />
    <SliderRow
      label="Detail"
      bind:value={editor.edits.detail.sharpen_detail}
      min={0}
      max={100}
      step={1}
      defaultValue={25}
      disabled={sharpenInactive}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      onPreviewStart={() => editor.onPreview('sharpen_detail')}
      onPreviewEnd={editor.endPreview}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Masking"
      bind:value={editor.edits.detail.sharpen_masking}
      min={0}
      max={100}
      step={1}
      disabled={sharpenInactive}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      onPreviewStart={() => editor.onPreview('sharpen_mask')}
      onPreviewEnd={editor.endPreview}
      format={(v: number) => v.toFixed(0)}
    />
  </div>
  <div class="flex flex-col gap-2.5 py-3">
    <div class="flex items-center justify-between">
      <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Noise Reduction</div>
      <button
        type="button"
        class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
        title="Reset Noise Reduction"
        aria-label="Reset Noise Reduction"
        onclick={resetNr}
      >
        <Icon path={mdiRestore} size={14} />
      </button>
    </div>
    <SliderRow
      label="Luminance"
      bind:value={editor.edits.detail.luma_nr_amount}
      min={0}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Detail"
      bind:value={editor.edits.detail.luma_nr_detail}
      min={0}
      max={100}
      step={1}
      defaultValue={50}
      disabled={lumaNrInactive}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Contrast"
      bind:value={editor.edits.detail.luma_nr_contrast}
      min={0}
      max={100}
      step={1}
      disabled={lumaNrInactive}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
  </div>
  <div class="flex flex-col gap-2.5 pt-3">
    <SliderRow
      label="Color"
      bind:value={editor.edits.detail.color_nr_amount}
      min={0}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Detail"
      bind:value={editor.edits.detail.color_nr_detail}
      min={0}
      max={100}
      step={1}
      defaultValue={50}
      disabled={colorNrInactive}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Smoothness"
      bind:value={editor.edits.detail.color_nr_smoothness}
      min={0}
      max={100}
      step={1}
      defaultValue={50}
      disabled={colorNrInactive}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
  </div>
</div>
