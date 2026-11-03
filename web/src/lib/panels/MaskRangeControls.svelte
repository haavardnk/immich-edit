<script lang="ts">
  import SliderRow from '#lib/components/editor/controls/SliderRow.svelte';
  import {
    COLOR_SOFTNESS,
    COLOR_TOLERANCE,
    LUMA_MAX,
    LUMA_MIN,
    LUMA_SOFTNESS
  } from '#lib/editor/params/maskComponents.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { MaskComponent } from '#lib/types/edits.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiEyedropperVariant } from '@mdi/js';

  let { layerId, component }: { layerId: string; component: MaskComponent } = $props();

  const luma = $derived(component.kind.kind === 'luma_range' ? component.kind : null);
  const color = $derived(component.kind.kind === 'color_range' ? component.kind : null);
  const softness = $derived(luma?.softness ?? color?.softness ?? LUMA_SOFTNESS.defaultValue);

  function commit(): void {
    void editor.commitMasks();
  }

  function updateLuma(field: 'min' | 'max' | 'softness', value: number): void {
    if (!luma) return;
    const next = { ...luma, [field]: value };
    if (field === 'min') next.min = Math.min(value, luma.max);
    if (field === 'max') next.max = Math.max(value, luma.min);
    editor.updateMaskComponentKind(layerId, component.id, next, true);
  }

  function updateColor(field: 'tolerance' | 'softness', value: number): void {
    if (!color) return;
    editor.updateMaskComponentKind(layerId, component.id, { ...color, [field]: value }, true);
  }

  function colorCss(rgb: [number, number, number]): string {
    const channels = rgb.map((value) => Math.round(Math.max(0, Math.min(1, value)) * 255));
    return `rgb(${channels[0]} ${channels[1]} ${channels[2]})`;
  }
</script>

{#if luma}
  <div class="mt-1 flex flex-col gap-1">
    <SliderRow
      label="Min"
      value={luma.min}
      min={LUMA_MIN.min}
      max={LUMA_MIN.max}
      step={LUMA_MIN.step}
      defaultValue={LUMA_MIN.defaultValue}
      onLive={(value: number) => updateLuma('min', value)}
      onCommit={commit}
      format={LUMA_MIN.format}
    />
    <SliderRow
      label="Max"
      value={luma.max}
      min={LUMA_MAX.min}
      max={LUMA_MAX.max}
      step={LUMA_MAX.step}
      defaultValue={LUMA_MAX.defaultValue}
      onLive={(value: number) => updateLuma('max', value)}
      onCommit={commit}
      format={LUMA_MAX.format}
    />
    <SliderRow
      label="Softness"
      value={softness}
      min={LUMA_SOFTNESS.min}
      max={LUMA_SOFTNESS.max}
      step={LUMA_SOFTNESS.step}
      defaultValue={LUMA_SOFTNESS.defaultValue}
      onLive={(value: number) => updateLuma('softness', value)}
      onCommit={commit}
      format={LUMA_SOFTNESS.format}
    />
  </div>
{/if}

{#if color}
  <div class="mt-1 flex flex-col gap-1">
    <div class="flex h-7 items-center justify-between">
      <span class="text-[11px] text-dark/65">Sample</span>
      <div class="flex items-center gap-1">
        <span
          class="h-4 w-4 rounded-sm ring-1 ring-light-950/10"
          style="background-color: {colorCss(color.sample_rgb)}"
        ></span>
        <Button
          type="button"
          size="tiny"
          variant="ghost"
          color={editor.colorPicker ? 'primary' : 'secondary'}
          class="size-6 p-0 {editor.colorPicker ? 'bg-light-950/10' : ''}"
          title={editor.colorPicker ? 'Cancel eyedropper' : 'Pick color from image'}
          aria-pressed={editor.colorPicker !== null}
          aria-label={editor.colorPicker ? 'Cancel eyedropper' : 'Pick color from image'}
          onclick={() => {
            if (editor.colorPicker) editor.cancelColorPicker();
            else editor.beginColorPicker(layerId, component.id);
          }}
        >
          <Icon icon={mdiEyedropperVariant} size="14px" aria-hidden="true" />
        </Button>
      </div>
    </div>
    <SliderRow
      label="Tolerance"
      value={color.tolerance}
      min={COLOR_TOLERANCE.min}
      max={COLOR_TOLERANCE.max}
      step={COLOR_TOLERANCE.step}
      defaultValue={COLOR_TOLERANCE.defaultValue}
      onLive={(value: number) => updateColor('tolerance', value)}
      onCommit={commit}
      format={COLOR_TOLERANCE.format}
    />
    <SliderRow
      label="Softness"
      value={softness}
      min={COLOR_SOFTNESS.min}
      max={COLOR_SOFTNESS.max}
      step={COLOR_SOFTNESS.step}
      defaultValue={COLOR_SOFTNESS.defaultValue}
      onLive={(value: number) => updateColor('softness', value)}
      onCommit={commit}
      format={COLOR_SOFTNESS.format}
    />
  </div>
{/if}
