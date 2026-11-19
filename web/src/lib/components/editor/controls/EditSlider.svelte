<script lang="ts">
  import SliderRow from './SliderRow.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { PreviewMode } from '#lib/api/preview.ts';
  import type { SliderRange } from '#lib/editor/params/types.ts';

  let {
    label,
    commitAction,
    value = $bindable(),
    range,
    defaultValue,
    gradient,
    disabled = false,
    previewMode,
    onLive = editor.onLive,
    onLabelTap,
    setting
  }: {
    label: string;
    commitAction: string;
    value: number;
    range: SliderRange;
    defaultValue?: number;
    gradient?: string;
    disabled?: boolean;
    previewMode?: PreviewMode;
    onLive?: (value: number) => void;
    onLabelTap?: () => void;
    setting?: string;
  } = $props();
</script>

<SliderRow
  {label}
  {commitAction}
  bind:value
  {range}
  {defaultValue}
  {gradient}
  {disabled}
  {onLive}
  {onLabelTap}
  {setting}
  onCommit={editor.onCommit}
  onPreviewStart={previewMode ? () => editor.onPreview(previewMode) : undefined}
  onPreviewEnd={previewMode ? editor.endPreview : undefined}
/>
