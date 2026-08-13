<script lang="ts">
  import SliderRow from './SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import type { PreviewMode } from '$lib/api/preview';

  let {
    label,
    commitAction,
    value = $bindable(),
    min,
    max,
    step = 1,
    defaultValue = 0,
    disabled = false,
    format = (v: number): string => v.toFixed(0),
    gradient,
    previewMode,
    onLive = editor.onLive
  }: {
    label: string;
    commitAction: string;
    value: number;
    min: number;
    max: number;
    step?: number;
    defaultValue?: number;
    disabled?: boolean;
    format?: (v: number) => string;
    gradient?: string;
    previewMode?: PreviewMode;
    onLive?: (value: number) => void;
  } = $props();
</script>

<SliderRow
  {label}
  {commitAction}
  bind:value
  {min}
  {max}
  {step}
  {defaultValue}
  {disabled}
  {format}
  {gradient}
  {onLive}
  onCommit={editor.onCommit}
  onPreviewStart={previewMode ? () => editor.onPreview(previewMode) : undefined}
  onPreviewEnd={previewMode ? editor.endPreview : undefined}
/>
