<script lang="ts" module>
  import type { MaskComponent, MaskLayer } from '#lib/types/edits.ts';

  export type TouchTool =
    | { kind: 'shape'; layer: MaskLayer; comp: MaskComponent; index: number; label: string }
    | { kind: 'click'; layer: MaskLayer | null }
    | { kind: 'polygon'; layer: MaskLayer | null };

  const ON_PHOTO = new Set(['linear', 'radial', 'polygon']);
  const CLICK_KINDS = new Set(['click', 'box']);

  export function onPhotoShape(comp: MaskComponent): boolean {
    if (comp.kind.kind === 'brush') return !comp.generated || CLICK_KINDS.has(comp.generated.kind);
    return ON_PHOTO.has(comp.kind.kind);
  }
</script>

<script lang="ts">
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import MaskClickTool from '#lib/panels/MaskClickTool.svelte';
  import MaskPolygonDraftControls from '#lib/panels/MaskPolygonDraftControls.svelte';
  import MaskShapeControls from '#lib/panels/MaskShapeControls.svelte';
  import { ADD_MODES, kindIcon, patchShape } from '#lib/panels/maskTools.ts';
  import { clickPointCount } from '#lib/panels/maskClickPill.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { MaskComponentMode } from '#lib/types/edits.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button, Icon, IconButton, Switch } from '@immich/ui';
  import { mdiAutoFix, mdiDotsHorizontal, mdiGestureTap, mdiVectorPolygon } from '@mdi/js';

  let { tool, onmenu }: { tool: TouchTool; onmenu: (componentId: string) => void } = $props();

  const icon = $derived.by(() => {
    if (tool.kind === 'click') return mdiGestureTap;
    if (tool.kind === 'polygon') return mdiVectorPolygon;
    return tool.comp.generated ? mdiAutoFix : kindIcon(tool.comp.kind);
  });
  const title = $derived(
    tool.kind === 'shape' ? tool.label : tool.kind === 'click' ? 'Click to select' : 'Polygon'
  );
  const subtitle = $derived.by(() => {
    const owner = tool.layer?.name ?? 'New mask';
    if (tool.kind === 'click') return `${owner} · ${countOf(clickPointCount(), 'dot')}`;
    if (tool.kind === 'polygon') {
      return `${owner} · ${countOf(editor.polygonDraft?.points.length ?? 0, 'corner')}`;
    }
    const total = tool.layer.components.length;
    return tool.index === 0
      ? `${owner} · base shape`
      : `${owner} · shape ${tool.index + 1} of ${total}`;
  });
  const gradient = $derived(
    tool.kind === 'shape' && tool.comp.kind.kind !== 'brush' ? tool.comp : null
  );
  const modes = ADD_MODES.map((entry) => ({
    value: entry.mode,
    label: entry.label,
    icon: entry.icon
  }));

  function done(): void {
    if (tool.kind === 'polygon') {
      void editor.finishPolygon();
      return;
    }
    editor.stopClickTool();
    if (tool.kind === 'shape') editor.setActiveMaskComponent(null);
  }

  function setMode(mode: MaskComponentMode): void {
    if (tool.kind === 'shape') patchShape(tool.layer.id, tool.comp.id, { mode });
  }
</script>

<section
  class="flex min-h-0 flex-1 flex-col"
  aria-label="{title} tool"
  data-testid="touch-shape-tool"
>
  <header class="flex h-14 shrink-0 items-center gap-3 border-b border-hairline ps-4 pe-3">
    <Icon {icon} size="20px" class="shrink-0 text-primary" aria-hidden="true" />
    <div class="flex min-w-0 flex-1 flex-col">
      <span class="truncate text-[15px] font-medium text-dark">{title}</span>
      <span class="truncate text-[12px] text-dark/65">{subtitle}</span>
    </div>
    {#if tool.kind === 'shape'}
      <IconButton
        size="medium"
        variant="ghost"
        color="secondary"
        class="size-10 shrink-0"
        icon={mdiDotsHorizontal}
        aria-label="{title} actions"
        title=""
        onclick={() => onmenu(tool.comp.id)}
      />
    {/if}
    <Button
      type="button"
      size="medium"
      shape="round"
      color="primary"
      class="h-10 shrink-0 px-5"
      disabled={tool.kind === 'polygon' && (editor.polygonDraft?.points.length ?? 0) < 3}
      onclick={done}
    >
      Done
    </Button>
  </header>
  <div
    class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto overscroll-contain px-4 py-2 scrollbar-hidden"
  >
    {#if tool.kind === 'click'}
      <MaskClickTool layerId={tool.layer?.id ?? null} pending />
    {:else if tool.kind === 'polygon'}
      <MaskPolygonDraftControls done={false} />
    {:else}
      {#if gradient && tool.index > 0}
        <SegmentedChoice
          options={modes}
          value={gradient.mode}
          label="Shape mode"
          fill
          onChange={setMode}
        />
      {/if}
      <MaskShapeControls
        layerId={tool.layer.id}
        component={tool.comp}
        brushSize={false}
        clickRefine={editor.clickTool.active || !!tool.comp.generated}
      />
      {#if gradient}
        <label class="flex h-11 items-center justify-between gap-3 text-[14px] text-dark">
          <span>Invert <span class="text-[12px] text-dark/65">use the outside</span></span>
          <Switch
            aria-label="Invert"
            checked={gradient.invert}
            onCheckedChange={(invert: boolean) => {
              if (tool.kind === 'shape') patchShape(tool.layer.id, tool.comp.id, { invert });
            }}
          />
        </label>
      {/if}
    {/if}
  </div>
</section>
