<script lang="ts">
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import EditSlider from '$lib/components/editor/controls/EditSlider.svelte';
  import SectionHeader from '$lib/components/editor/controls/SectionHeader.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { NEUTRAL_DETAIL, neutralSharpenAmount } from '$lib/types/edits';
  import { Tooltip } from '@immich/ui';

  const isRaw = $derived(editor.meta?.is_raw ?? false);
  const defaultSharpen = $derived(neutralSharpenAmount(isRaw));
  const sharpenAmount = $derived(editor.edits.detail.sharpen_amount ?? defaultSharpen);
  const sharpenInactive = $derived(sharpenAmount === 0);
  const lumaNrInactive = $derived(editor.edits.detail.luma_nr_amount === 0);
  const colorNrInactive = $derived(editor.edits.detail.color_nr_amount === 0);
  const sharpenModified = $derived(
    editor.edits.detail.capture_sharpen !== NEUTRAL_DETAIL.capture_sharpen ||
      editor.edits.detail.sharpen_amount !== NEUTRAL_DETAIL.sharpen_amount ||
      editor.edits.detail.sharpen_radius !== NEUTRAL_DETAIL.sharpen_radius ||
      editor.edits.detail.sharpen_detail !== NEUTRAL_DETAIL.sharpen_detail ||
      editor.edits.detail.sharpen_masking !== NEUTRAL_DETAIL.sharpen_masking
  );
  const nrModified = $derived(!lumaNrInactive || !colorNrInactive);

  function setSharpenAmount(v: number): void {
    editor.edits.detail.sharpen_amount = v === defaultSharpen ? null : v;
    editor.onLive();
  }

  function onToggleCaptureSharpen(checked: boolean): void {
    editor.edits.detail.capture_sharpen = checked;
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

<div class="flex flex-col divide-y divide-dark/10">
  <div class="flex flex-col gap-1 pb-1.5">
    <SectionHeader title="Sharpening" modified={sharpenModified} onReset={resetSharpen} />
    <Tooltip
      text={isRaw
        ? 'Compensates for sensor and anti-aliasing filter blur'
        : 'Only available for raw files'}
    >
      {#snippet child({ props })}
        <CheckboxRow
          label="Capture Sharpening"
          checked={editor.edits.detail.capture_sharpen}
          disabled={!isRaw}
          onChange={onToggleCaptureSharpen}
          {...props}
        />
      {/snippet}
    </Tooltip>
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
  <div class="flex flex-col gap-1 py-1.5">
    <SectionHeader title="Noise Reduction" modified={nrModified} onReset={resetNr} />
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
  <div class="flex flex-col gap-1 pt-1.5">
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
