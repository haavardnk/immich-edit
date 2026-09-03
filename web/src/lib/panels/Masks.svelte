<script lang="ts">
  import Notice from '$lib/components/Notice.svelte';
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import MaskAdjustments from './MaskAdjustments.svelte';
  import MaskBrushControls from './MaskBrushControls.svelte';
  import MaskGeneratedControls from './MaskGeneratedControls.svelte';
  import MaskRangeControls from './MaskRangeControls.svelte';
  import MaskPolygonControls from './MaskPolygonControls.svelte';
  import MaskClickRefine from './MaskClickRefine.svelte';
  import MaskComponentRow from './MaskComponentRow.svelte';
  import MaskLayerRow from './MaskLayerRow.svelte';
  import MaskToolMenu from './MaskToolMenu.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { maskModels } from '$lib/stores/maskModels.svelte';
  import {
    N_MAX_MASK_LAYERS,
    N_MAX_COMPONENTS_PER_LAYER,
    N_MAX_TOTAL_COMPONENTS,
    type MaskComponent,
    type MaskComponentMode,
    type MaskLayer
  } from '$lib/types/edits';
  import { generatedLabel, kindLabel, numberRepeats } from '$lib/types/masks';
  import { mergeProps } from '$lib/utils/mergeProps';
  import { Button, Icon, IconButton, Tooltip } from '@immich/ui';
  import {
    mdiPlus,
    mdiMinus,
    mdiSetCenter,
    mdiEye,
    mdiEyeOff,
    mdiChevronDown,
    mdiChevronRight
  } from '@mdi/js';

  let layerMenu = $state<'header' | 'empty' | null>(null);
  let componentMenu = $state<MaskComponentMode | null>(null);
  let refineOverride = $state<Record<string, boolean>>({});

  const layers = $derived(editor.edits.masks);
  const active = $derived<MaskLayer | null>(
    editor.activeLayerId ? (layers.find((l) => l.id === editor.activeLayerId) ?? null) : null
  );
  const activeComp = $derived<MaskComponent | null>(
    active && editor.activeMaskComponentId
      ? (active.components.find((c) => c.id === editor.activeMaskComponentId) ?? null)
      : null
  );
  const cap = $derived(editor.maskCapacityFor(editor.activeLayerId));
  const compLabels = $derived(
    active
      ? numberRepeats(
          active.components.map((c) =>
            c.generated ? generatedLabel(c.generated.kind) : kindLabel(c.kind)
          )
        )
      : []
  );
  const refineOpen = $derived(
    active
      ? (refineOverride[active.id] ?? (active.components.length === 0 || activeComp !== null))
      : false
  );
  const featherValue = $derived(
    activeComp && (activeComp.kind.kind === 'linear' || activeComp.kind.kind === 'radial')
      ? activeComp.kind.feather
      : 0.5
  );

  $effect(() => {
    void maskModels.load();
  });

  function toggleRefine(): void {
    if (!active) return;
    refineOverride = { ...refineOverride, [active.id]: !refineOpen };
  }

  function onFeatherLive(v: number): void {
    if (active && activeComp) editor.setMaskComponentFeather(active.id, activeComp.id, v);
  }

  function openAddComponent(mode: MaskComponentMode, open: boolean): void {
    if (open && active) refineOverride = { ...refineOverride, [active.id]: true };
    componentMenu = open ? mode : null;
  }
</script>

<div class="flex flex-col gap-1">
  <div class="flex h-8 items-center justify-between border-b border-hairline px-1">
    <div class="text-[9px] font-semibold uppercase text-dark/65">
      Masks ({layers.length}/{N_MAX_MASK_LAYERS})
    </div>
    <div class="flex items-center gap-1">
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        class="bg-transparent hover:bg-white/6"
        icon={editor.maskOverlayVisible ? mdiEye : mdiEyeOff}
        title={editor.maskOverlayVisible ? 'Hide mask overlays' : 'Show mask overlays'}
        aria-label="Toggle mask overlays"
        aria-pressed={editor.maskOverlayVisible}
        onclick={editor.toggleMaskOverlay}
      />
      <div class="relative inline-flex items-center">
        <MaskToolMenu
          open={layerMenu === 'header'}
          align="end"
          target={{ layerId: null, mode: 'add' }}
          onOpenChange={(v) => (layerMenu = v ? 'header' : null)}
        >
          {#snippet trigger(menuProps)}
            <Tooltip
              text={cap.layersFull
                ? `You have used all ${N_MAX_MASK_LAYERS} masks. Delete one to add another.`
                : cap.totalFull
                  ? `You have used all ${N_MAX_TOTAL_COMPONENTS} shapes across every mask. Delete one to add another.`
                  : 'Create a new mask'}
            >
              {#snippet child({ props })}
                <Button
                  type="button"
                  size="tiny"
                  variant="ghost"
                  color="secondary"
                  class="h-8 bg-transparent hover:bg-white/6"
                  leadingIcon={mdiPlus}
                  aria-label="New mask"
                  disabled={cap.layersFull || cap.totalFull}
                  {...mergeProps(props, menuProps)}
                >
                  New
                </Button>
              {/snippet}
            </Tooltip>
          {/snippet}
        </MaskToolMenu>
      </div>
    </div>
  </div>

  {#if editor.maskError}
    <Notice message={`Could not build that mask. ${editor.maskError}`} class="mx-1">
      <div class="flex items-center gap-1 shrink-0">
        <Button
          type="button"
          size="tiny"
          color="danger"
          disabled={editor.maskGenerating}
          onclick={() => void editor.retryMask()}
        >
          Try again
        </Button>
        <Button
          type="button"
          size="tiny"
          variant="ghost"
          color="secondary"
          onclick={editor.dismissMaskError}
        >
          Dismiss
        </Button>
      </div>
    </Notice>
  {/if}

  {#if layers.length === 0}
    <div class="flex justify-center px-2 py-2">
      <MaskToolMenu
        open={layerMenu === 'empty'}
        align="start"
        target={{ layerId: null, mode: 'add' }}
        onOpenChange={(v) => (layerMenu = v ? 'empty' : null)}
      >
        {#snippet trigger(props)}
          <Button type="button" size="tiny" color="primary" leadingIcon={mdiPlus} {...props}>
            Create a mask
          </Button>
        {/snippet}
      </MaskToolMenu>
    </div>
  {:else}
    <div class="flex flex-col gap-0.5">
      {#each layers as layer, li (layer.id)}
        <MaskLayerRow {layer} index={li} total={layers.length} {cap} />
      {/each}
    </div>
  {/if}

  {#if layers.length > 0 && !active}
    <div class="px-1.5 pt-1 text-[11px] text-dark/65">Select a mask to adjust it.</div>
  {/if}

  {#if active}
    <div class="mt-1 flex flex-col gap-1 rounded-md border border-hairline bg-white/3 p-1">
      <div class="flex items-center gap-1 px-1">
        <Button
          type="button"
          size="tiny"
          variant="ghost"
          color="secondary"
          class="flex-1 min-w-0 justify-start"
          title={refineOpen
            ? 'Hide the shapes that build this mask'
            : 'Show the shapes that build this mask'}
          aria-expanded={refineOpen}
          onclick={toggleRefine}
        >
          <Icon
            icon={refineOpen ? mdiChevronDown : mdiChevronRight}
            size="14px"
            class="opacity-40 shrink-0"
            aria-hidden="true"
          />
          <span class="truncate text-[10px] font-medium text-dark/65">
            Refine ({active.components.length}/{N_MAX_COMPONENTS_PER_LAYER})
          </span>
        </Button>
        <Button
          type="button"
          size="tiny"
          variant={active.invert ? 'filled' : 'ghost'}
          color={active.invert ? 'primary' : 'secondary'}
          class="shrink-0"
          title={active.invert
            ? 'Editing everything outside the shapes. Click to go back.'
            : 'Edit everything outside the shapes instead'}
          aria-pressed={active.invert}
          disabled={active.components.length === 0}
          onclick={() => void editor.toggleMaskLayerInvert(active.id)}
        >
          Invert
        </Button>
      </div>

      {#if refineOpen}
        <div class="relative flex h-7 items-center gap-1 px-1">
          <MaskToolMenu
            open={componentMenu === 'add'}
            align="start"
            heading
            target={{ layerId: active.id, mode: 'add' }}
            onOpenChange={(v) => openAddComponent('add', v)}
          >
            {#snippet trigger(menuProps)}
              <Tooltip text="Add another shape to this mask">
                {#snippet child({ props })}
                  <Button
                    type="button"
                    size="tiny"
                    variant="ghost"
                    color="secondary"
                    leadingIcon={mdiPlus}
                    disabled={cap.componentsFull || cap.totalFull}
                    {...mergeProps(props, menuProps)}
                  >
                    Add
                  </Button>
                {/snippet}
              </Tooltip>
            {/snippet}
          </MaskToolMenu>
          <MaskToolMenu
            open={componentMenu === 'subtract'}
            align="start"
            heading
            target={{ layerId: active.id, mode: 'subtract' }}
            onOpenChange={(v) => openAddComponent('subtract', v)}
          >
            {#snippet trigger(menuProps)}
              <Tooltip text="Cut a shape out of this mask">
                {#snippet child({ props })}
                  <Button
                    type="button"
                    size="tiny"
                    variant="ghost"
                    color="secondary"
                    leadingIcon={mdiMinus}
                    disabled={cap.componentsFull || cap.totalFull || active.components.length === 0}
                    {...mergeProps(props, menuProps)}
                  >
                    Subtract
                  </Button>
                {/snippet}
              </Tooltip>
            {/snippet}
          </MaskToolMenu>
          <MaskToolMenu
            open={componentMenu === 'intersect'}
            align="start"
            heading
            target={{ layerId: active.id, mode: 'intersect' }}
            onOpenChange={(v) => openAddComponent('intersect', v)}
          >
            {#snippet trigger(menuProps)}
              <Tooltip text="Keep only where this shape overlaps the mask">
                {#snippet child({ props })}
                  <Button
                    type="button"
                    size="tiny"
                    variant="ghost"
                    color="secondary"
                    leadingIcon={mdiSetCenter}
                    disabled={cap.componentsFull || cap.totalFull || active.components.length === 0}
                    {...mergeProps(props, menuProps)}
                  >
                    Intersect
                  </Button>
                {/snippet}
              </Tooltip>
            {/snippet}
          </MaskToolMenu>
        </div>

        {#if cap.componentsFull || cap.totalFull}
          <div class="px-1 text-[10px] text-dark/65">
            {cap.componentsFull
              ? `This mask is full at ${N_MAX_COMPONENTS_PER_LAYER} shapes. Delete one to add another.`
              : `You have used all ${N_MAX_TOTAL_COMPONENTS} shapes across every mask. Delete one to add another.`}
          </div>
        {/if}

        {#if active.components.length === 0}
          <div class="px-1 py-1 text-[10px] italic text-dark/65">
            Empty mask. Use Add to pick a tool.
          </div>
        {:else}
          {#each active.components as comp, i (comp.id)}
            <MaskComponentRow
              layerId={active.id}
              {comp}
              index={i}
              total={active.components.length}
              label={compLabels[i] ?? ''}
            />
          {/each}
        {/if}

        {#if activeComp && (activeComp.kind.kind === 'linear' || activeComp.kind.kind === 'radial')}
          <div class="mt-1 flex flex-col gap-1">
            <SliderRow
              label="Feather"
              value={featherValue}
              min={0}
              max={1}
              step={0.01}
              defaultValue={0.5}
              onLive={onFeatherLive}
              onCommit={() => void editor.commitMasks()}
              format={(v: number) => v.toFixed(2)}
            />
          </div>
        {/if}

        {#if activeComp && activeComp.kind.kind === 'brush' && maskModels.clickInstalled}
          <MaskClickRefine layerId={active.id} />
        {/if}

        {#if activeComp?.generated}
          <MaskGeneratedControls layerId={active.id} component={activeComp} />
        {:else if activeComp && activeComp.kind.kind === 'brush'}
          <MaskBrushControls />
        {/if}

        {#if activeComp && (activeComp.kind.kind === 'luma_range' || activeComp.kind.kind === 'color_range')}
          <MaskRangeControls layerId={active.id} component={activeComp} />
        {/if}

        {#if activeComp && activeComp.kind.kind === 'polygon'}
          <MaskPolygonControls layerId={active.id} component={activeComp} />
        {/if}
      {/if}
    </div>

    <MaskAdjustments layerId={active.id} />
  {/if}
</div>
