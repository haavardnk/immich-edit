<script lang="ts">
  import { mergeProps } from 'bits-ui';
  import { Tooltip } from '@immich/ui';
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import ParamSlider from '#lib/components/editor/controls/ParamSlider.svelte';
  import SectionHeader from '#lib/components/editor/controls/SectionHeader.svelte';
  import {
    aiDenoiseDetail,
    aiDenoiseStrength,
    colorNr,
    colorNrDetail,
    colorNrSmoothness,
    lumaNr,
    lumaNrContrast,
    lumaNrDetail,
    noColorNr,
    noLumaNr
  } from '#lib/editor/params/detail.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import {
    AI_DENOISE_STRENGTH,
    aiDenoiseActive,
    NEUTRAL_DETAIL,
    neutraliseSection
  } from '#lib/types/edits.ts';
  import AiDenoiseStatus from './AiDenoiseStatus.svelte';

  const aiDenoiseOn = $derived(editor.edits.detail.ai_denoise);
  const modified = $derived(aiDenoiseOn || !noLumaNr(editor.edits) || !noColorNr(editor.edits));

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
      text={editor.isRaw
        ? 'Removes noise with a neural network, once per photo at full resolution'
        : 'Only available for raw files'}
    >
      {#snippet child({ props })}
        <CheckboxRow
          label="AI Denoise"
          checked={aiDenoiseOn}
          disabled={!editor.isRaw}
          onChange={onToggleAiDenoise}
          {...mergeProps(props, { tabindex: -1 })}
        />
      {/snippet}
    </Tooltip>
    <ParamSlider param={aiDenoiseStrength} />
    <ParamSlider param={aiDenoiseDetail} />
    {#if editor.isRaw && aiDenoiseActive(editor.edits.detail)}
      <AiDenoiseStatus status={editor.aiDenoise.status} onRetry={() => editor.aiDenoise.retry()} />
    {/if}
  </div>
</div>
<div class="flex flex-col gap-1 py-1.5">
  <ParamSlider param={lumaNr} />
  <ParamSlider param={lumaNrDetail} />
  <ParamSlider param={lumaNrContrast} />
</div>
<div class="flex flex-col gap-1 pt-1.5">
  <ParamSlider param={colorNr} />
  <ParamSlider param={colorNrDetail} />
  <ParamSlider param={colorNrSmoothness} />
</div>
