<script lang="ts">
  import SliderRow from '#lib/components/editor/controls/SliderRow.svelte';
  import {
    commitRetouchSetting,
    liveRetouch,
    RETOUCH_RANGES,
    retouchValue
  } from '#lib/editor/retouch/settings.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { formatBrushSize } from '#lib/utils/brushSize.ts';

  let { size = true }: { size?: boolean } = $props();
</script>

{#if size}
  <SliderRow
    label="Size"
    value={retouchValue('size')}
    range={RETOUCH_RANGES.size}
    onLive={(v: number) => liveRetouch('size', v)}
    onCommit={commitRetouchSetting}
    format={(v: number) => formatBrushSize(v, editor.meta)}
  />
{/if}
<SliderRow
  label="Hardness"
  value={retouchValue('hardness')}
  range={RETOUCH_RANGES.hardness}
  onLive={(v: number) => liveRetouch('hardness', v)}
  onCommit={commitRetouchSetting}
/>
<SliderRow
  label="Opacity"
  value={retouchValue('opacity')}
  range={RETOUCH_RANGES.opacity}
  onLive={(v: number) => liveRetouch('opacity', v)}
  onCommit={commitRetouchSetting}
/>
