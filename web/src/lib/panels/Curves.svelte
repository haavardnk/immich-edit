<script lang="ts">
  import ResetButton from '#lib/components/editor/controls/ResetButton.svelte';
  import { commitSectionReset } from '#lib/components/editor/controls/paramReset.ts';
  import { CURVES } from '#lib/editor/params/basic.ts';
  import { held } from '#lib/input/modifiers.svelte.ts';
  import { keyLabel } from '#lib/shortcuts/chord.ts';
  import { curveEditor } from '#lib/stores/curveEditor.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { CURVE_CHANNELS, identityCurve, type CurveChannel } from '#lib/types/edits.ts';
  import { Tooltip } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';
  import CurvesCanvas from './curves/CurvesCanvas.svelte';
  import CurvePointBar from './curves/CurvePointBar.svelte';
  import { CHANNEL_LABELS } from './curves/curves';

  let selected = $state<number | null>(null);

  const channel = $derived(curveEditor.channel);

  const CHANNEL_SHORT: Record<CurveChannel, string> = {
    composite: 'RGB',
    r: 'R',
    g: 'G',
    b: 'B',
    luma: 'L'
  };

  const CHANNEL_DOT: Partial<Record<CurveChannel, string>> = {
    r: 'var(--color-curve-red)',
    g: 'var(--color-curve-green)',
    b: 'var(--color-curve-blue)'
  };

  function resetActive(): void {
    editor.edits.basic.curves = { ...editor.edits.basic.curves, [channel]: identityCurve() };
    selected = null;
    void editor.onCommit(`Reset Curve ${CHANNEL_LABELS[channel]}`);
  }

  function resetAll(): void {
    selected = null;
    commitSectionReset(CURVES);
  }
</script>

<div class="flex flex-col gap-1.5">
  <div class="flex items-center gap-1">
    <RadioGroup.Root
      bind:value={() => channel, (v) => (curveEditor.channel = v as CurveChannel)}
      orientation="horizontal"
      aria-label="Curve channel"
      class="flex min-w-0 flex-1 gap-1 overflow-x-auto scrollbar-hidden touch:gap-1.5"
    >
      {#each CURVE_CHANNELS as ch (ch)}
        <Tooltip text={CHANNEL_LABELS[ch]}>
          {#snippet child({ props })}
            <RadioGroup.Item
              value={ch}
              class="flex h-6 shrink-0 items-center gap-1.5 rounded-full px-2 text-[11px] font-medium whitespace-nowrap outline-none focus-visible:ring-2 focus-visible:ring-primary aria-checked:bg-primary/22 aria-checked:text-light-950 aria-[checked=false]:bg-ghost aria-[checked=false]:text-dark/80 touch:h-8 touch:px-3 touch:text-[13.5px] touch:font-normal"
              aria-label="Edit {CHANNEL_LABELS[ch]} curve"
              {...props}
            >
              {#if CHANNEL_DOT[ch]}
                <span
                  aria-hidden="true"
                  class="hidden size-2 rounded-full touch:block"
                  style:background={CHANNEL_DOT[ch]}
                ></span>
              {/if}
              <span class="touch:hidden">{CHANNEL_SHORT[ch]}</span>
              <span class="hidden touch:inline">{CHANNEL_LABELS[ch]}</span>
            </RadioGroup.Item>
          {/snippet}
        </Tooltip>
      {/each}
    </RadioGroup.Root>
    <ResetButton
      title="Reset {CHANNEL_LABELS[channel]} curve  —  {keyLabel(
        'Shift'
      )}-click to reset all channels"
      label="Reset {CHANNEL_LABELS[channel]} curve"
      onclick={(e) => (held('resetAll', e) ? resetAll() : resetActive())}
    />
  </div>

  <CurvesCanvas {channel} bind:selected />
  <CurvePointBar {channel} bind:selected />
</div>
