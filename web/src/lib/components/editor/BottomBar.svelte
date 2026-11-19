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
  class="@container relative shrink-0 bg-editor-chrome before:absolute before:inset-x-0 before:top-0 before:h-px before:bg-hairline"
>
  <div
    class="grid auto-rows-9 grid-cols-[minmax(0,1fr)_auto] items-center px-2 @xl:grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)]"
  >
    <div
      class="col-span-2 -my-1 -ms-1 flex min-w-0 items-center gap-2 overflow-hidden py-1 ps-1 @xl:col-span-1"
    >
      {#if cull}
        <DisabledReason
          locked={!cull.owned}
          reason={NOT_OWNER}
          class="-my-1 -ms-1 flex min-w-0 flex-1 items-center gap-2 overflow-hidden py-1 ps-1"
        >
          <div
            class="flex h-7 shrink-0 items-center gap-1 [&_button:not([data-label])]:text-light-950/55 [&_button:not([data-label]):hover]:text-light-950/90"
          >
            <CullControls {...cull} starSize={14} actions={editorCullActions} />
          </div>
          <div class="flex min-w-0 flex-1 overflow-hidden">
            <TagsStrip disabled={!cull.owned} />
          </div>
        </DisabledReason>
      {/if}
    </div>

    <SaveStatus />

    <div class="col-start-2 flex min-w-0 items-center justify-end gap-1 @xl:col-start-3">
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
  </div>
</nav>
