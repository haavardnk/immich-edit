<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';

  const sharpenInactive = $derived(editor.edits.detail.sharpen_amount === 0);
  const lumaNrInactive = $derived(editor.edits.detail.luma_nr_amount === 0);
  const colorNrInactive = $derived(editor.edits.detail.color_nr_amount === 0);
</script>

<div class="flex flex-col divide-y divide-white/5">
  <div class="flex flex-col gap-2.5 pb-3">
    <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Sharpening</div>
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
    <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Noise Reduction</div>
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
