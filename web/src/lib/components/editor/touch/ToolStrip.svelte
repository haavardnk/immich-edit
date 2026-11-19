<script lang="ts">
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { RETOUCH_MODE_LABEL, RETOUCH_MODES } from '#lib/editor/retouch/modes.ts';
  import { selectedStroke } from '#lib/editor/retouch/settings.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { paintInput } from '#lib/stores/editor/paintInput.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Icon, IconButton } from '@immich/ui';
  import { mdiBandage, mdiChevronUp, mdiLayersTripleOutline } from '@mdi/js';

  const BRUSH_MODES = [
    { value: 'paint', label: 'Paint' },
    { value: 'erase', label: 'Erase' }
  ] as const;

  const retouch = $derived(ui.editorTab === 'retouch');
  const retouchMode = $derived(selectedStroke()?.stroke.mode ?? editor.retouchTool.mode);
  const layer = $derived(editor.activeLayer());
  const detail = $derived(
    retouch ? countOf(editor.edits.retouch.length, 'spot') : (layer?.name ?? 'No mask selected')
  );
</script>

<section
  aria-label="Tool"
  class="flex h-13 shrink-0 items-center gap-2.5 rounded-t-2xl border-t border-hairline bg-editor-panel ps-3.5 pe-2"
>
  <Icon
    icon={retouch ? mdiBandage : mdiLayersTripleOutline}
    size="20px"
    class="shrink-0 text-primary"
    aria-hidden="true"
  />
  <span class="text-body font-medium text-light-950">{retouch ? 'Retouch' : 'Masks'}</span>
  <span class="min-w-0 truncate text-label text-dark/65">{detail}</span>
  <span class="flex-1"></span>
  {#if retouch}
    <SegmentedChoice
      appearance="mini"
      label="Retouch tool"
      value={retouchMode}
      options={RETOUCH_MODES.map((value) => ({ value, label: RETOUCH_MODE_LABEL[value] }))}
      onChange={editor.setRetouchMode}
    />
  {:else if paintInput.brushActive}
    <SegmentedChoice
      appearance="mini"
      label="Brush mode"
      value={editor.brushTool.mode}
      options={BRUSH_MODES}
      onChange={(mode) => editor.setBrushTool({ mode })}
    />
  {/if}
  <IconButton
    size="medium"
    variant="ghost"
    color="secondary"
    icon={mdiChevronUp}
    title=""
    aria-label="Show tool settings"
    onclick={ui.togglePanels}
  />
</section>
