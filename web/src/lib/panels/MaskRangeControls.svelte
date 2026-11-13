<script lang="ts">
  import SliderRow from '#lib/components/editor/controls/SliderRow.svelte';
  import {
    COLOR_SOFTNESS,
    COLOR_TOLERANCE,
    LUMA_SOFTNESS
  } from '#lib/editor/params/maskComponents.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { MaskComponent } from '#lib/types/edits.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiArrowRight, mdiClose, mdiEyedropperVariant } from '@mdi/js';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { keyLabel } from '#lib/shortcuts/chord.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import MaskLumaRange from './MaskLumaRange.svelte';

  let { layerId, component }: { layerId: string; component: MaskComponent } = $props();

  const luma = $derived(component.kind.kind === 'luma_range' ? component.kind : null);
  const color = $derived(component.kind.kind === 'color_range' ? component.kind : null);
  const softness = $derived(luma?.softness ?? color?.softness ?? LUMA_SOFTNESS.defaultValue);
  const picker = $derived(
    editor.rangePicker?.componentId === component.id ? editor.rangePicker : null
  );
  const samples = $derived(picker?.samples.length ?? 0);
  const noun = $derived(luma ? 'luminance' : 'color');
  const help = $derived(
    layout.mode === 'touch'
      ? 'Drag to a spot on the photo, then lift.'
      : `Drag on the photo to sample an area; ${keyLabel('Shift')}-click adds a sample. The range widens to cover every sample.`
  );

  function commit(): void {
    void editor.commitMasks();
  }

  function updateLuma(patch: Partial<{ min: number; max: number; softness: number }>): void {
    if (!luma) return;
    editor.updateMaskComponentKind(layerId, component.id, { ...luma, ...patch }, true);
  }

  function updateColor(field: 'tolerance' | 'softness', value: number): void {
    if (!color) return;
    editor.updateMaskComponentKind(layerId, component.id, { ...color, [field]: value }, true);
  }

  function toggleSample(): void {
    if (picker) editor.cancelRangePicker();
    else editor.beginRangePicker(layerId, component.id);
  }

  function colorCss(rgb: readonly [number, number, number]): string {
    const channels = rgb.map((value) => Math.round(Math.max(0, Math.min(1, value)) * 255));
    return `rgb(${channels[0]} ${channels[1]} ${channels[2]})`;
  }
</script>

{#snippet swatch(rgb: readonly [number, number, number], testid: string)}
  <span
    data-testid={testid}
    class="size-4 shrink-0 rounded-sm ring-1 ring-light-950/10 touch:size-6"
    style="background-color: {colorCss(rgb)}"
  ></span>
{/snippet}

<div class="mt-1 flex flex-col gap-1">
  <div class="flex min-h-7 items-center gap-2">
    <Button
      type="button"
      size="tiny"
      variant={picker ? 'filled' : 'ghost'}
      color={picker ? 'primary' : 'secondary'}
      class="gap-1 px-2 touch:h-10 touch:px-3 touch:text-[13px]"
      aria-label="Sample {noun} from the photo"
      aria-pressed={picker !== null}
      onclick={toggleSample}
    >
      <Icon icon={mdiEyedropperVariant} size="14px" aria-hidden="true" />
      Sample
    </Button>
    {#if color}
      {@render swatch(color.sample_rgb, 'range-swatch')}
      {#if picker?.preview}
        <Icon icon={mdiArrowRight} size="12px" class="text-dark/65" aria-hidden="true" />
        {@render swatch(picker.preview, 'range-swatch-preview')}
      {/if}
    {/if}
    {#if picker && samples > 0}
      <span class="ms-auto text-[11px] text-dark/65 tabular-nums">{countOf(samples, 'sample')}</span
      >
      <Button
        type="button"
        size="tiny"
        variant="ghost"
        color="secondary"
        leadingIcon={mdiClose}
        onclick={editor.clearRangeSamples}
      >
        Clear
      </Button>
    {/if}
  </div>
  {#if picker}
    <p class="text-[10px] text-dark/65 touch:text-[12px]">{help}</p>
  {/if}

  {#if luma}
    <MaskLumaRange
      min={luma.min}
      max={luma.max}
      softness={luma.softness}
      onLive={([min, max]) => updateLuma({ min, max })}
      onCommit={commit}
    />
    <SliderRow
      label="Softness"
      value={softness}
      range={LUMA_SOFTNESS}
      onLive={(value: number) => updateLuma({ softness: value })}
      onCommit={commit}
    />
  {/if}

  {#if color}
    <SliderRow
      label="Tolerance"
      value={color.tolerance}
      range={COLOR_TOLERANCE}
      onLive={(value: number) => updateColor('tolerance', value)}
      onCommit={commit}
    />
    <SliderRow
      label="Softness"
      value={softness}
      range={COLOR_SOFTNESS}
      onLive={(value: number) => updateColor('softness', value)}
      onCommit={commit}
    />
  {/if}
</div>
