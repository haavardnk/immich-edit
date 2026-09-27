<script lang="ts">
  import ResetButton from '$lib/components/editor/controls/ResetButton.svelte';
  import { compactSegmentedSwatchItemClass } from '$lib/components/editor/controls/segmentedControl';
  import { editor } from '$lib/stores/editor.svelte';
  import { keyLabel } from '$lib/shortcuts/chord';
  import { Tooltip } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';
  import {
    CURVE_CHANNELS,
    identityCurve,
    neutralCurves,
    type CurveChannel
  } from '$lib/types/edits';
  import CurvesCanvas from './curves/CurvesCanvas.svelte';
  import { CHANNEL_LABELS } from './curves/curves';

  let activeChannel: CurveChannel = $state('composite');
  let canvas = $state<ReturnType<typeof CurvesCanvas>>();

  const channelSwatchStyle: Record<CurveChannel, string> = {
    composite: 'background: var(--gradient-curve-rgb)',
    r: 'background: var(--color-curve-red)',
    g: 'background: var(--color-curve-green)',
    b: 'background: var(--color-curve-blue)',
    luma: 'background: var(--gradient-curve-luma)'
  };

  function selectChannel(ch: CurveChannel): void {
    if (activeChannel === ch) return;
    activeChannel = ch;
    canvas?.clearSelection();
  }

  function resetActive(): void {
    editor.edits.basic.curves = { ...editor.edits.basic.curves, [activeChannel]: identityCurve() };
    canvas?.clearSelection();
    void editor.onCommit(`Reset Curve ${CHANNEL_LABELS[activeChannel]}`);
  }

  function resetAll(): void {
    editor.edits.basic.curves = neutralCurves();
    canvas?.clearSelection();
    void editor.onCommit('Reset Curves');
  }
</script>

<div class="flex flex-col gap-1">
  <RadioGroup.Root
    bind:value={() => activeChannel, (v) => selectChannel(v as CurveChannel)}
    orientation="horizontal"
    aria-label="Curve channel"
    class="grid grid-cols-5 gap-0.5"
  >
    {#each CURVE_CHANNELS as ch (ch)}
      <Tooltip text={CHANNEL_LABELS[ch]}>
        {#snippet child({ props })}
          <RadioGroup.Item
            value={ch}
            class="{compactSegmentedSwatchItemClass} flex items-center justify-center overflow-hidden"
            style={channelSwatchStyle[ch]}
            aria-label="Edit {CHANNEL_LABELS[ch]} curve"
            {...props}
          />
        {/snippet}
      </Tooltip>
    {/each}
  </RadioGroup.Root>

  <div class="flex h-6 items-center justify-between">
    <div class="text-[11px] text-dark/65">{CHANNEL_LABELS[activeChannel]}</div>
    <ResetButton
      title="Reset {CHANNEL_LABELS[activeChannel]} curve  —  {keyLabel(
        'Shift'
      )}-click to reset all channels"
      label="Reset {CHANNEL_LABELS[activeChannel]} curve"
      onclick={(e) => (e.shiftKey ? resetAll() : resetActive())}
    />
  </div>

  <CurvesCanvas bind:this={canvas} channel={activeChannel} />
</div>
