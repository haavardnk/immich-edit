<script lang="ts">
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import ElementTooltip from '#lib/components/ElementTooltip.svelte';
  import ParamSlider from '#lib/components/editor/controls/ParamSlider.svelte';
  import SectionHeader from '#lib/components/editor/controls/SectionHeader.svelte';
  import { commitSectionReset } from '#lib/components/editor/controls/paramReset.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import {
    captureCornerBoost,
    SHARPENING,
    sharpenAmount,
    sharpenDetail,
    sharpenMasking,
    sharpenRadius
  } from '#lib/editor/params/detail.ts';
  import { NEUTRAL_DETAIL } from '#lib/types/edits.ts';
  import { mergeProps } from 'bits-ui';
  import NoiseReductionSection from './detail/NoiseReductionSection.svelte';

  const sharpenModified = $derived(
    editor.edits.detail.capture_sharpen !== NEUTRAL_DETAIL.capture_sharpen ||
      editor.edits.detail.capture_corner_boost !== NEUTRAL_DETAIL.capture_corner_boost ||
      editor.edits.detail.sharpen_amount !== NEUTRAL_DETAIL.sharpen_amount ||
      editor.edits.detail.sharpen_radius !== NEUTRAL_DETAIL.sharpen_radius ||
      editor.edits.detail.sharpen_detail !== NEUTRAL_DETAIL.sharpen_detail ||
      editor.edits.detail.sharpen_masking !== NEUTRAL_DETAIL.sharpen_masking
  );

  function onToggleCaptureSharpen(checked: boolean): void {
    editor.edits.detail.capture_sharpen = checked;
    if (!checked) editor.edits.detail.capture_corner_boost = NEUTRAL_DETAIL.capture_corner_boost;
    void editor.onCommit('Capture Sharpening');
  }
</script>

<div class="flex flex-col divide-y divide-dark/10">
  <div class="flex flex-col gap-1 pb-1.5">
    <SectionHeader
      title="Sharpening"
      section="sharpening"
      modified={sharpenModified}
      onReset={() => commitSectionReset(SHARPENING)}
    />
    <ElementTooltip
      text={editor.isRaw
        ? 'Compensates for sensor and anti-aliasing filter blur'
        : 'Only available for raw files'}
    >
      {#snippet child({ props })}
        <CheckboxRow
          label="Capture Sharpening"
          checked={editor.edits.detail.capture_sharpen}
          disabled={!editor.isRaw}
          onChange={onToggleCaptureSharpen}
          {...mergeProps(props, { tabindex: -1 })}
        />
      {/snippet}
    </ElementTooltip>
    <ParamSlider param={captureCornerBoost} />
  </div>
  <div class="flex flex-col gap-1 py-1.5">
    <ParamSlider param={sharpenAmount} />
    <ParamSlider param={sharpenRadius} />
    <ParamSlider param={sharpenDetail} />
    <ParamSlider param={sharpenMasking} />
  </div>
  <NoiseReductionSection />
</div>
