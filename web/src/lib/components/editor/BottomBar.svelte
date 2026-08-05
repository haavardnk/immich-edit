<script lang="ts">
  import ToolbarButton from '$lib/components/ToolbarButton.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { hint } from '$lib/keybinds';
  import { editor } from '$lib/stores/editor.svelte';
  import RatingControl from './RatingControl.svelte';
  import TagsStrip from './TagsStrip.svelte';
  import SaveStatus from './SaveStatus.svelte';
  import { mdiMagnifyMinusOutline, mdiMagnifyPlusOutline, mdiFitToScreenOutline, mdiFullscreen, mdiFullscreenExit } from '@mdi/js';

  const hasAsset = $derived(editor.asset != null);
</script>

<div class="flex items-center justify-between gap-3 px-3 py-1 bg-immich-dark-bg/80 backdrop-blur-sm border-t border-white/5">
  <div class="flex items-center gap-3 min-w-0">
    {#if hasAsset}
      <RatingControl />
      <div class="min-w-0">
        <TagsStrip />
      </div>
      <SaveStatus />
    {/if}
  </div>

  <div class="flex items-center gap-0.5">
    <Popover
      open={ui.zoomPopoverOpen}
      anchor="top"
      align="end"
      onClose={ui.closeZoomPopover}
      contentClass="p-2"
    >
      {#snippet trigger()}
        <ToolbarButton
          variant="text"
          label={`${ui.zoom}%`}
          title="Zoom"
          active={ui.zoomPopoverOpen}
          onclick={ui.toggleZoomPopover}
        />
      {/snippet}
      {#snippet children()}
        <div class="flex items-center gap-2">
          <ToolbarButton path={mdiMagnifyMinusOutline} size={16} title="Zoom Out" onclick={ui.zoomOut} />
          <input
            type="range"
            min="25"
            max="400"
            step="5"
            value={ui.zoom}
            oninput={(e: Event) => ui.setZoom(Number((e.target as HTMLInputElement).value))}
            class="w-28 h-1 accent-immich-dark-primary"
          />
          <ToolbarButton path={mdiMagnifyPlusOutline} size={16} title="Zoom In" onclick={ui.zoomIn} />
          <ToolbarButton path={mdiFitToScreenOutline} size={16} title={hint('Fit to screen', 'zoomToggle')} onclick={ui.zoomFit} />
        </div>
      {/snippet}
    </Popover>
    <ToolbarButton
      path={ui.fullscreen ? mdiFullscreenExit : mdiFullscreen}
      size={18}
      title={hint('Fullscreen', 'fullscreen')}
      onclick={ui.toggleFullscreen}
    />
  </div>
</div>

