<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { mdiMagnifyMinusOutline, mdiMagnifyPlusOutline } from '@mdi/js';
</script>

<div class="flex items-center justify-end px-3 py-1 bg-immich-dark-bg/80 backdrop-blur-sm border-t border-white/5">
  <div class="flex items-center gap-1.5">
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
      title="Fit to Screen"
      onclick={ui.zoomFit}
    >
      {ui.zoom}%
    </button>
  </div>
</div>
