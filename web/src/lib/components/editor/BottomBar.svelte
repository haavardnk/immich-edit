<script lang="ts">
  import ZoomPopover from '$lib/components/ZoomPopover.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { hint } from '$lib/shortcuts/labels';
  import { editor } from '$lib/stores/editor.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { NOT_OWNER, ownsAsset } from '$lib/browse/ownership';
  import DisabledReason from '$lib/components/DisabledReason.svelte';
  import RatingControl from './RatingControl.svelte';
  import TagsStrip from './TagsStrip.svelte';
  import SaveStatus from './SaveStatus.svelte';
  import { Button, IconButton } from '@immich/ui';
  import { mdiFilmstrip, mdiFullscreen, mdiFullscreenExit, mdiPaletteSwatchOutline } from '@mdi/js';

  const hasAsset = $derived(editor.asset != null);
  const owned = $derived(editor.asset ? ownsAsset(editor.asset) : false);
  const hasFilmstrip = $derived(browsing.visible.length > 0);
  const filmstripShown = $derived(!ui.editorFilmstripCollapsed && !ui.presetStrip);
</script>

<nav
  aria-label="Editor status and view controls"
  class="relative grid h-9 shrink-0 grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center border-t border-hairline bg-editor-chrome px-2"
>
  <div class="-my-1 -ms-1 flex min-w-0 items-center gap-2 overflow-hidden py-1 ps-1">
    {#if hasAsset}
      <DisabledReason
        locked={!owned}
        reason={NOT_OWNER}
        class="-my-1 -ms-1 flex min-w-0 items-center gap-2 overflow-hidden py-1 ps-1"
      >
        <RatingControl />
        <div class="min-w-0 overflow-hidden">
          <TagsStrip disabled={!owned} />
        </div>
      </DisabledReason>
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
    <Button
      size="tiny"
      variant="ghost"
      color={ui.presetStrip ? 'primary' : 'secondary'}
      leadingIcon={mdiPaletteSwatchOutline}
      title={hint(ui.presetStrip ? 'Hide presets' : 'Show presets', 'presetStrip')}
      aria-label={ui.presetStrip ? 'Hide presets' : 'Show presets'}
      aria-pressed={ui.presetStrip}
      onclick={ui.togglePresetStrip}>Presets</Button
    >
    <Button
      size="tiny"
      variant="ghost"
      color={hasFilmstrip && filmstripShown ? 'primary' : 'secondary'}
      leadingIcon={mdiFilmstrip}
      title={!hasFilmstrip
        ? 'No filmstrip available'
        : hint(filmstripShown ? 'Hide filmstrip' : 'Show filmstrip', 'filmstrip')}
      aria-label={!hasFilmstrip
        ? 'No filmstrip available'
        : filmstripShown
          ? 'Hide filmstrip'
          : 'Show filmstrip'}
      aria-pressed={filmstripShown}
      disabled={!hasFilmstrip}
      onclick={ui.toggleEditorFilmstrip}>Filmstrip</Button
    >
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
