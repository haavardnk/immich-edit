<script lang="ts">
  import MaskAdjustments from '#lib/panels/MaskAdjustments.svelte';
  import MaskClickTool from '#lib/panels/MaskClickTool.svelte';
  import MaskShapeControls from '#lib/panels/MaskShapeControls.svelte';
  import { useMaskPanel } from '#lib/panels/maskPanel.svelte.ts';
  import { shapeLabels } from '#lib/panels/maskShapeText.ts';
  import {
    ADD_MODES,
    addModeBlocked,
    clickPending,
    type MaskTarget
  } from '#lib/panels/maskTools.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { maskedEditsIsZero, N_MAX_COMPONENTS_PER_LAYER } from '#lib/types/edits.ts';
  import { Button, Icon } from '@immich/ui';
  import { Tabs } from 'bits-ui';
  import MaskAddSheet from './MaskAddSheet.svelte';
  import MaskLayerCards from './MaskLayerCards.svelte';
  import MaskLayerSheet from './MaskLayerSheet.svelte';
  import MaskShapeSheet from './MaskShapeSheet.svelte';
  import TouchShapeList from './TouchShapeList.svelte';
  import MaskPolygonDraftControls from '#lib/panels/MaskPolygonDraftControls.svelte';
  import { layerLimitText, shapeLimitText } from '#lib/panels/maskLimits.ts';
  import { mdiInformationOutline } from '@mdi/js';
  import TouchShapeTool, { onPhotoShape, type TouchTool } from './TouchShapeTool.svelte';

  let { stacked = false }: { stacked?: boolean } = $props();

  type Tab = 'shapes' | 'adjust';

  let tab = $state<Tab>('shapes');
  let adding = $state<MaskTarget | null>(null);
  let layerMenu = $state<string | null>(null);
  let shapeMenu = $state<string | null>(null);

  useMaskPanel();

  const active = $derived(editor.activeLayer());
  const activeComp = $derived(editor.activeMaskComponent());
  const labels = $derived(shapeLabels(active));
  const pending = $derived(clickPending());
  const draft = $derived(editor.polygonDraft);
  const adjusted = $derived(active ? !maskedEditsIsZero(active.edits) : false);
  const shapeMenuLabel = $derived(
    active ? (labels[active.components.findIndex((comp) => comp.id === shapeMenu)] ?? '') : ''
  );
  const layerCap = $derived(editor.maskCapacityFor(null));
  const layerLimit = $derived(
    layerCap.layersFull || (layerCap.totalFull && !active) ? layerLimitText(layerCap) : null
  );
  const shapeLimit = $derived(active ? shapeLimitText(editor.maskCapacityFor(active.id)) : null);
  const tool = $derived.by<TouchTool | null>(() => {
    if (stacked) return null;
    if (draft) {
      return {
        kind: 'polygon',
        layer: editor.edits.masks.find((layer) => layer.id === draft.layerId) ?? null
      };
    }
    if (pending) {
      return {
        kind: 'click',
        layer: active && editor.clickTool.layerId === active.id ? active : null
      };
    }
    if (!active || !activeComp || !onPhotoShape(activeComp)) return null;
    const index = active.components.indexOf(activeComp);
    return { kind: 'shape', layer: active, comp: activeComp, index, label: labels[index] ?? '' };
  });
  const inline = $derived(activeComp && (stacked || !onPhotoShape(activeComp)) ? activeComp : null);
  const tabClass =
    'flex shrink-0 items-center gap-1.5 border-b-2 text-[13.5px] whitespace-nowrap outline-none focus-visible:text-primary data-[state=active]:border-primary data-[state=active]:font-medium data-[state=active]:text-light-950 data-[state=inactive]:border-transparent data-[state=inactive]:text-dark/65';
</script>

{#snippet limit(text: string, testid: string)}
  <p
    data-testid={testid}
    class="mx-3 my-1.5 flex items-start gap-2 rounded-xl border border-hairline bg-ghost px-3 py-2.5 text-[13px] text-dark/80"
  >
    <Icon icon={mdiInformationOutline} size="16px" class="mt-px shrink-0" aria-hidden="true" />
    {text}
  </p>
{/snippet}

{#snippet shapes()}
  {#if active}
    {#if active.components.length === 0}
      <p class="px-4 py-3 text-[13px] text-dark/65">This mask is empty. Add a shape below.</p>
    {:else}
      <TouchShapeList layer={active} {labels} onmenu={(id) => (shapeMenu = id)} />
    {/if}
    {#if inline}
      <div class="flex flex-col gap-1 px-4 py-2">
        <MaskShapeControls layerId={active.id} component={inline} />
      </div>
    {:else if stacked && draft?.layerId === active.id}
      <div class="px-4 py-2"><MaskPolygonDraftControls /></div>
    {:else if pending && editor.clickTool.layerId === active.id}
      <div class="px-4 py-2"><MaskClickTool layerId={active.id} pending /></div>
    {/if}
  {/if}
{/snippet}

{#snippet addButtons()}
  {#if active}
    <div class="grid shrink-0 grid-cols-3 gap-2 px-3 py-1.5">
      {#each ADD_MODES as entry (entry.mode)}
        <Button
          type="button"
          size="medium"
          variant="ghost"
          color="secondary"
          class="h-10 min-w-0 gap-1 rounded-xl bg-ghost px-1.5 text-[13px] font-normal"
          leadingIcon={entry.icon}
          disabled={addModeBlocked(entry.mode, active.id, active.components.length)}
          onclick={() => (adding = { layerId: active.id, mode: entry.mode })}
        >
          {entry.label}
        </Button>
      {/each}
    </div>
    {#if shapeLimit}
      {@render limit(shapeLimit, 'mask-shape-limit')}
    {/if}
  {/if}
{/snippet}

{#snippet empty()}
  {#if pending && editor.clickTool.layerId === null}
    <div class="px-4 py-2"><MaskClickTool layerId={null} pending /></div>
  {:else if stacked && draft && draft.layerId === null}
    <div class="px-4 py-2"><MaskPolygonDraftControls /></div>
  {:else if editor.edits.masks.length === 0}
    <p class="px-4 py-3 text-[13px] text-dark/65">Tap + to add a mask.</p>
  {:else if !active}
    <p class="px-4 py-3 text-[13px] text-dark/65">Tap a mask to edit it.</p>
  {/if}
{/snippet}

{#if tool}
  <TouchShapeTool {tool} onmenu={(id) => (shapeMenu = id)} />
{:else}
  <div
    class="flex min-h-0 flex-1 flex-col {stacked
      ? 'overflow-y-auto overscroll-contain scrollbar-hidden'
      : ''}"
  >
    <MaskLayerCards
      onadd={() => (adding = { layerId: null, mode: 'add' })}
      onmenu={(id) => (layerMenu = id)}
    />
    {#if layerLimit}
      {@render limit(layerLimit, 'mask-layer-limit')}
    {/if}
    {@render empty()}
    {#if active && stacked}
      <div class="flex h-9 items-center justify-between border-y border-hairline px-4">
        <span class="text-[11px] font-medium tracking-wider text-dark/65 uppercase">Shapes</span>
        <span class="text-[12px] text-dark/65 tabular-nums">
          {active.components.length} of {N_MAX_COMPONENTS_PER_LAYER}
        </span>
      </div>
      {@render shapes()}
      {@render addButtons()}
      <div class="px-3 pb-3"><MaskAdjustments layerId={active.id} /></div>
    {:else if active}
      <Tabs.Root
        value={tab}
        onValueChange={(next) => (tab = next as Tab)}
        class="flex min-h-0 flex-1 flex-col"
      >
        <div class="flex shrink-0 items-center border-y border-hairline pe-4">
          <Tabs.List
            aria-label="Mask views"
            class="flex h-10 min-w-0 flex-1 items-stretch gap-5 px-4"
          >
            <Tabs.Trigger value="shapes" class={tabClass}>Shapes</Tabs.Trigger>
            <Tabs.Trigger value="adjust" class={tabClass}>
              Adjust
              {#if adjusted}
                <span aria-hidden="true" class="size-1.5 rounded-full bg-primary"></span>
              {/if}
            </Tabs.Trigger>
          </Tabs.List>
          <span class="shrink-0 text-[12px] text-dark/65 tabular-nums">
            {active.components.length} of {N_MAX_COMPONENTS_PER_LAYER}
          </span>
        </div>
        <Tabs.Content
          value="shapes"
          class="min-h-0 flex-1 overflow-y-auto overscroll-contain scrollbar-hidden"
        >
          {@render shapes()}
        </Tabs.Content>
        <Tabs.Content
          value="adjust"
          class="min-h-0 flex-1 overflow-y-auto overscroll-contain px-3 pb-2 scrollbar-hidden"
        >
          <MaskAdjustments layerId={active.id} />
        </Tabs.Content>
        {#if tab === 'shapes'}
          {@render addButtons()}
        {/if}
      </Tabs.Root>
    {/if}
  </div>
{/if}

<MaskAddSheet target={adding} onclose={() => (adding = null)} />
<MaskLayerSheet layerId={layerMenu} onclose={() => (layerMenu = null)} />
{#if active}
  <MaskShapeSheet
    layer={active}
    componentId={shapeMenu}
    label={shapeMenuLabel}
    onclose={() => (shapeMenu = null)}
  />
{/if}
