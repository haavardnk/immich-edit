<script lang="ts">
  import EditSlider from '$lib/components/editor/controls/EditSlider.svelte';
  import SectionHeader from '$lib/components/editor/controls/SectionHeader.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { NEUTRAL_DETAIL, neutralSharpenAmount } from '$lib/types/edits';

  const isRaw = $derived(editor.meta?.is_raw ?? false);
  const defaultSharpen = $derived(neutralSharpenAmount(isRaw));
  const sharpenAmount = $derived(editor.edits.detail.sharpen_amount ?? defaultSharpen);
  const sharpenInactive = $derived(sharpenAmount === 0);
  const lumaNrInactive = $derived(editor.edits.detail.luma_nr_amount === 0);
  const colorNrInactive = $derived(editor.edits.detail.color_nr_amount === 0);

  function setSharpenAmount(v: number): void {
    editor.edits.detail.sharpen_amount = v === defaultSharpen ? null : v;
    editor.onLive();
  }

  function onToggleCaptureSharpen(e: Event): void {
    editor.edits.detail.capture_sharpen = (e.currentTarget as HTMLInputElement).checked;
    editor.onCommit('Capture Sharpening');
  }

  function resetSharpen(): void {
    editor.edits.detail.capture_sharpen = NEUTRAL_DETAIL.capture_sharpen;
    editor.edits.detail.sharpen_amount = NEUTRAL_DETAIL.sharpen_amount;
    editor.edits.detail.sharpen_radius = NEUTRAL_DETAIL.sharpen_radius;
    editor.edits.detail.sharpen_detail = NEUTRAL_DETAIL.sharpen_detail;
    editor.edits.detail.sharpen_masking = NEUTRAL_DETAIL.sharpen_masking;
    editor.onCommit('Reset Sharpening');
  }

  function resetNr(): void {
    editor.edits.detail.luma_nr_amount = NEUTRAL_DETAIL.luma_nr_amount;
    editor.edits.detail.luma_nr_detail = NEUTRAL_DETAIL.luma_nr_detail;
    editor.edits.detail.luma_nr_contrast = NEUTRAL_DETAIL.luma_nr_contrast;
    editor.edits.detail.color_nr_amount = NEUTRAL_DETAIL.color_nr_amount;
    editor.edits.detail.color_nr_detail = NEUTRAL_DETAIL.color_nr_detail;
    editor.edits.detail.color_nr_smoothness = NEUTRAL_DETAIL.color_nr_smoothness;
    editor.onCommit('Reset Noise Reduction');
  }
</script>

<div class="flex flex-col divide-y divide-white/5">
  <div class="flex flex-col gap-2.5 pb-3">
    <SectionHeader title="Sharpening" onReset={resetSharpen} />
    <label
      class="flex items-center gap-2 text-[11px] text-immich-dark-fg/80"
      class:cursor-pointer={isRaw}
      class:opacity-40={!isRaw}
      title={isRaw
        ? 'Compensates for sensor and anti-aliasing filter blur'
        : 'Only available for raw files'}
    >
      <input
        type="checkbox"
        class="checkbox checkbox-xs checkbox-primary"
        checked={editor.edits.detail.capture_sharpen}
        disabled={!isRaw}
        onchange={onToggleCaptureSharpen}
      />
      Capture Sharpening
    </label>
    <EditSlider
      label="Amount"
      commitAction="Sharpen Amount"
      value={sharpenAmount}
      min={0}
      max={150}
      defaultValue={defaultSharpen}
      onLive={setSharpenAmount}
    />
    <EditSlider
      label="Radius"
      commitAction="Sharpen Radius"
      bind:value={editor.edits.detail.sharpen_radius}
      min={0.5}
      max={3.0}
      step={0.1}
      defaultValue={1.0}
      disabled={sharpenInactive}
      previewMode="sharpen_radius"
      format={(v: number) => v.toFixed(1)}
    />
    <EditSlider
      label="Detail"
      commitAction="Sharpen Detail"
      bind:value={editor.edits.detail.sharpen_detail}
      min={0}
      max={100}
      defaultValue={25}
      disabled={sharpenInactive}
      previewMode="sharpen_detail"
    />
    <EditSlider
      label="Masking"
      commitAction="Sharpen Masking"
      bind:value={editor.edits.detail.sharpen_masking}
      min={0}
      max={100}
      disabled={sharpenInactive}
      previewMode="sharpen_mask"
    />
  </div>
  <div class="flex flex-col gap-2.5 py-3">
    <SectionHeader title="Noise Reduction" onReset={resetNr} />
    <EditSlider
      label="Luminance"
      commitAction="Luminance NR"
      bind:value={editor.edits.detail.luma_nr_amount}
      min={0}
      max={100}
    />
    <EditSlider
      label="Detail"
      commitAction="Luminance NR Detail"
      bind:value={editor.edits.detail.luma_nr_detail}
      min={0}
      max={100}
      defaultValue={50}
      disabled={lumaNrInactive}
    />
    <EditSlider
      label="Contrast"
      commitAction="Luminance NR Contrast"
      bind:value={editor.edits.detail.luma_nr_contrast}
      min={0}
      max={100}
      disabled={lumaNrInactive}
    />
  </div>
  <div class="flex flex-col gap-2.5 pt-3">
    <EditSlider
      label="Color"
      commitAction="Color NR"
      bind:value={editor.edits.detail.color_nr_amount}
      min={0}
      max={100}
    />
    <EditSlider
      label="Detail"
      commitAction="Color NR Detail"
      bind:value={editor.edits.detail.color_nr_detail}
      min={0}
      max={100}
      defaultValue={50}
      disabled={colorNrInactive}
    />
    <EditSlider
      label="Smoothness"
      commitAction="Color NR Smoothness"
      bind:value={editor.edits.detail.color_nr_smoothness}
      min={0}
      max={100}
      defaultValue={50}
      disabled={colorNrInactive}
    />
  </div>
</div>
