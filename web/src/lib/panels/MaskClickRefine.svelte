<script lang="ts">
  import { compactSegmentedControlClass } from '$lib/components/editor/controls/segmentedControl';
  import { editor } from '$lib/stores/editor.svelte';
  import { Button } from '@immich/ui';
  import { refineWith, stopClickTool } from './maskTools';

  let { layerId }: { layerId: string } = $props();

  const active = $derived(editor.clickTool.active && editor.clickTool.layerId === layerId);
</script>

<div class="mt-1 flex h-7 items-center justify-between gap-1.5">
  <div class="text-[10px] font-medium text-dark/65">Click refine</div>
  <div class="flex items-center gap-1">
    <div class="{compactSegmentedControlClass} flex text-[10px]">
      <Button
        type="button"
        size="tiny"
        variant={active && !editor.clickTool.negative ? 'filled' : 'ghost'}
        color={active && !editor.clickTool.negative ? 'primary' : 'secondary'}
        title="Click the photo to add that area to this shape"
        onclick={() => refineWith(layerId, false)}
      >
        Add
      </Button>
      <Button
        type="button"
        size="tiny"
        variant={active && editor.clickTool.negative ? 'filled' : 'ghost'}
        color={active && editor.clickTool.negative ? 'primary' : 'secondary'}
        title="Click the photo to cut that area out of this shape"
        onclick={() => refineWith(layerId, true)}
      >
        Remove
      </Button>
    </div>
    {#if active}
      <Button type="button" size="tiny" variant="ghost" color="secondary" onclick={stopClickTool}>
        Done
      </Button>
    {/if}
  </div>
</div>
{#if editor.maskGenerating}
  <div class="text-[10px] text-dark/65">Working…</div>
{/if}
