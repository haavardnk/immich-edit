<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import type { MaskComponent } from '$lib/types/edits';
  import { Button, Icon } from '@immich/ui';
  import { mdiEyedropperVariant } from '@mdi/js';

  let { layerId, component }: { layerId: string; component: MaskComponent } = $props();

  const luma = $derived(component.kind.kind === 'luma_range' ? component.kind : null);
  const color = $derived(component.kind.kind === 'color_range' ? component.kind : null);
  const softness = $derived(luma?.softness ?? color?.softness ?? 0.1);

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
      min={0}
      max={1}
      step={0.01}
      defaultValue={0.25}
      onLive={(value: number) => updateLuma('min', value)}
      onCommit={commit}
      format={(value: number) => value.toFixed(2)}
    />
    <SliderRow
      label="Max"
      value={luma.max}
      min={0}
      max={1}
      step={0.01}
      defaultValue={0.75}
      onLive={(value: number) => updateLuma('max', value)}
      onCommit={commit}
      format={(value: number) => value.toFixed(2)}
    />
    <SliderRow
      label="Softness"
      value={softness}
      min={0}
      max={1}
      step={0.01}
      defaultValue={0.1}
      onLive={(value: number) => updateLuma('softness', value)}
      onCommit={commit}
      format={(value: number) => value.toFixed(2)}
    />
  </div>
{/if}

{#if color}
  <div class="mt-1 flex flex-col gap-1">
    <div class="flex h-7 items-center justify-between">
      <span class="text-[11px] text-dark/65">Sample</span>
      <div class="flex items-center gap-1">
        <span
          class="h-4 w-4 rounded-sm ring-1 ring-white/10"
          style="background-color: {colorCss(color.sample_rgb)}"
        ></span>
        <Button
          type="button"
          size="tiny"
          variant="ghost"
          color={editor.colorPicker ? 'primary' : 'secondary'}
          class="size-6 p-0 {editor.colorPicker ? 'bg-white/10' : ''}"
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
      min={0}
      max={1}
      step={0.01}
      defaultValue={0.1}
      onLive={(value: number) => updateColor('tolerance', value)}
      onCommit={commit}
      format={(value: number) => value.toFixed(2)}
    />
    <SliderRow
      label="Softness"
      value={softness}
      min={0}
      max={1}
      step={0.01}
      defaultValue={0.05}
      onLive={(value: number) => updateColor('softness', value)}
      onCommit={commit}
      format={(value: number) => value.toFixed(2)}
    />
  </div>
{/if}
