<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import RatingControl from './RatingControl.svelte';
  import TagsStrip from './TagsStrip.svelte';
  import SaveStatus from './SaveStatus.svelte';
  import { mdiMagnifyMinusOutline, mdiMagnifyPlusOutline } from '@mdi/js';

  const hasAsset = $derived(editor.asset != null);
</script>

<div class="grid grid-cols-[minmax(0,1fr)_auto_auto] items-center gap-3 px-3 py-1 bg-immich-dark-bg/80 backdrop-blur-sm border-t border-white/5">
  <div class="min-w-0">
    {#if hasAsset}
      <TagsStrip />
    {/if}
  </div>

  <div class="flex items-center gap-2">
    {#if hasAsset}
      <RatingControl />
      <SaveStatus />
    {/if}
  </div>

  <div class="flex items-center gap-1.5 justify-self-end">
    <button
      class="btn btn-ghost btn-xs btn-square"
      title="Zoom Out"
      onclick={ui.zoomOut}
    >
      <Icon path={mdiMagnifyMinusOutline} size={16} />
    </button>
    <input
      type="range"
      min="25"
      max="400"
      step="5"
      value={ui.zoom}
      oninput={(e: Event) => ui.setZoom(Number((e.target as HTMLInputElement).value))}
      class="range range-xs w-24"
    />
    <button
      class="btn btn-ghost btn-xs btn-square"
      title="Zoom In"
      onclick={ui.zoomIn}
    >
      <Icon path={mdiMagnifyPlusOutline} size={16} />
    </button>
    <button
      class="btn btn-ghost btn-xs min-w-14 font-mono text-xs"
      title="Fit to Screen (Space / Z)"
      onclick={ui.zoomFit}
    >
      {ui.zoom}%
    </button>
  </div>
</div>

