<script lang="ts">
  import ZoomPopover from '#lib/components/ZoomPopover.svelte';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { NOT_OWNER } from '#lib/browse/ownership.ts';
  import { cullState } from '#lib/browse/cullState.ts';
  import { editorCullActions } from '#lib/commands/editor/culling.ts';
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import CullControls from '#lib/components/CullControls.svelte';
  import TagsStrip from '#lib/components/editor/TagsStrip.svelte';
  import SaveStatus from '#lib/components/editor/SaveStatus.svelte';
  import { Button } from '@immich/ui';
  import { mdiFilmstrip, mdiPaletteSwatchOutline } from '@mdi/js';

  const cull = $derived(editor.asset ? cullState(editor.asset) : null);
  const hasFilmstrip = $derived(browsing.visible.length > 0);
  const filmstripShown = $derived(!ui.editorFilmstripCollapsed && !ui.presetStrip);
</script>

<nav
  aria-label="Editor status and view controls"
  class="relative grid h-9 shrink-0 grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center border-t border-hairline bg-editor-chrome px-2"
>
  <div class="-my-1 -ms-1 flex min-w-0 items-center gap-2 overflow-hidden py-1 ps-1">
    {#if cull}
      <DisabledReason
        locked={!cull.owned}
        reason={NOT_OWNER}
        class="-my-1 -ms-1 flex min-w-0 items-center gap-2 overflow-hidden py-1 ps-1"
      >
        <div
          class="flex h-7 shrink-0 items-center gap-1 [&_button:not([data-label])]:text-light-950/55 [&_button:not([data-label]):hover]:text-light-950/90"
        >
          <CullControls {...cull} starSize={14} actions={editorCullActions} />
        </div>
        <div class="min-w-0 overflow-hidden">
          <TagsStrip disabled={!cull.owned} />
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
      class={ui.presetStrip ? 'bg-primary/10' : undefined}
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
      class={hasFilmstrip && filmstripShown ? 'bg-primary/10' : undefined}
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
  </div>
</nav>
