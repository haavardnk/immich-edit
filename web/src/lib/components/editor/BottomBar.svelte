<script lang="ts">
  import ZoomPopover from '$lib/components/ZoomPopover.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { hint } from '$lib/keybinds';
  import { editor } from '$lib/stores/editor.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import RatingControl from './RatingControl.svelte';
  import TagsStrip from './TagsStrip.svelte';
  import SaveStatus from './SaveStatus.svelte';
  import { IconButton } from '@immich/ui';
  import { mdiFilmstrip, mdiFullscreen, mdiFullscreenExit } from '@mdi/js';

  const hasAsset = $derived(editor.asset != null);
  const hasFilmstrip = $derived(browsing.assets.length > 0);
</script>

<nav
  aria-label="Editor status and view controls"
  class="relative grid h-9 shrink-0 grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center border-t border-hairline bg-editor-chrome px-2"
>
  <div class="flex min-w-0 items-center gap-2 overflow-hidden">
    {#if hasAsset}
      <RatingControl />
      <div class="min-w-0 flex-1 overflow-hidden">
        <TagsStrip />
      </div>
    {/if}
  </div>

  <SaveStatus />

  <div class="col-start-3 flex min-w-0 items-center justify-end gap-1">
    <ZoomPopover
      open={ui.metaPopover === 'zoom'}
      zoom={ui.zoom}
      fitZoom={ui.fitZoom}
      fitMode={ui.fitMode}
      onOpenChange={(v) => (v ? ui.openPopover('zoom') : ui.closePopover())}
      onZoom={ui.userZoom}
      onFit={ui.zoomFit}
    />
    <IconButton
      size="tiny"
      variant="ghost"
      color={hasFilmstrip && !ui.editorFilmstripCollapsed ? 'primary' : 'secondary'}
      icon={mdiFilmstrip}
      title={!hasFilmstrip
        ? 'No filmstrip available'
        : ui.editorFilmstripCollapsed
          ? 'Show filmstrip'
          : 'Hide filmstrip'}
      aria-label={!hasFilmstrip
        ? 'No filmstrip available'
        : ui.editorFilmstripCollapsed
          ? 'Show filmstrip'
          : 'Hide filmstrip'}
      aria-pressed={!ui.editorFilmstripCollapsed}
      disabled={!hasFilmstrip}
      onclick={ui.toggleEditorFilmstrip}
    />
    <IconButton
      size="tiny"
      variant="ghost"
      color="secondary"
      icon={ui.fullscreen ? mdiFullscreenExit : mdiFullscreen}
      title={hint('Fullscreen', 'fullscreen')}
      aria-label={hint('Fullscreen', 'fullscreen')}
      onclick={ui.toggleFullscreen}
    />
  </div>
</nav>
