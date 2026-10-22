<script lang="ts">
  import { mergeProps } from 'bits-ui';
  import { Tooltip } from '@immich/ui';
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import EditSlider from '$lib/components/editor/controls/EditSlider.svelte';
  import SectionHeader from '$lib/components/editor/controls/SectionHeader.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import {
    AI_DENOISE_STRENGTH,
    aiDenoiseActive,
    NEUTRAL_DETAIL,
    neutraliseSection
  } from '$lib/types/edits';
  import AiDenoiseStatus from './AiDenoiseStatus.svelte';

  const isRaw = $derived(editor.meta?.is_raw ?? false);
  const aiDenoiseOn = $derived(editor.edits.detail.ai_denoise);
  const lumaNrInactive = $derived(editor.edits.detail.luma_nr_amount === 0);
  const colorNrInactive = $derived(editor.edits.detail.color_nr_amount === 0);
  const modified = $derived(aiDenoiseOn || !lumaNrInactive || !colorNrInactive);

  function onToggleAiDenoise(checked: boolean): void {
    editor.edits.detail.ai_denoise = checked;
    if (!checked) {
      editor.edits.detail.ai_denoise_strength = AI_DENOISE_STRENGTH;
      editor.edits.detail.ai_denoise_detail = NEUTRAL_DETAIL.ai_denoise_detail;
    }
    void editor.onCommit('AI Denoise');
  }

  function reset(): void {
    editor.edits = neutraliseSection(editor.edits, 'noise_reduction');
    void editor.onCommit('Reset Noise Reduction');
  }
</script>

<div class="flex flex-col gap-1 py-1.5">
  <SectionHeader title="Noise Reduction" section="noise_reduction" {modified} onReset={reset} />
  <div role="group" aria-label="AI Denoise" class="flex flex-col gap-1">
    <Tooltip
      text={isRaw
        ? 'Removes noise with a neural network, once per photo at full resolution'
        : 'Only available for raw files'}
    >
      {#snippet child({ props })}
        <CheckboxRow
          label="AI Denoise"
          checked={aiDenoiseOn}
          disabled={!isRaw}
          onChange={onToggleAiDenoise}
          {...mergeProps(props, { tabindex: -1 })}
        />
      {/snippet}
    </Tooltip>
    <EditSlider
      label="Strength"
      commitAction="AI Denoise Strength"
      bind:value={editor.edits.detail.ai_denoise_strength}
      min={0}
      max={100}
      defaultValue={AI_DENOISE_STRENGTH}
      disabled={!isRaw || !aiDenoiseOn}
    />
    <EditSlider
      label="Detail"
      commitAction="AI Denoise Detail"
      bind:value={editor.edits.detail.ai_denoise_detail}
      min={0}
      max={100}
      disabled={!isRaw || !aiDenoiseOn}
    />
    {#if isRaw && aiDenoiseActive(editor.edits.detail)}
      <AiDenoiseStatus status={editor.aiDenoise.status} onRetry={() => editor.aiDenoise.retry()} />
    {/if}
  </div>
</div>
<div class="flex flex-col gap-1 py-1.5">
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
