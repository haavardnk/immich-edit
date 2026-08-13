<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
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
  import {
    mdiPlus,
    mdiMinus,
    mdiSetCenter,
    mdiEye,
    mdiEyeOff,
    mdiChevronDown,
    mdiChevronRight
  } from '@mdi/js';

  let addLayerOpen = $state(false);
  let addComponentOpen = $state(false);
  let pendingMode = $state<MaskComponentMode>('add');
  let addLayerBtn = $state<HTMLButtonElement | undefined>(undefined);
  let addComponentBtn = $state<HTMLButtonElement | undefined>(undefined);
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

  function toggleAddLayer(): void {
    addLayerOpen = !addLayerOpen;
  }

  function openAddComponent(mode: MaskComponentMode): void {
    pendingMode = mode;
    if (active) refineOverride = { ...refineOverride, [active.id]: true };
    addComponentOpen = !addComponentOpen;
  }
</script>

<div class="flex flex-col gap-2">
  <div class="flex items-center justify-between px-1.5">
    <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">
      Masks ({layers.length}/{N_MAX_MASK_LAYERS})
    </div>
    <div class="flex items-center gap-1">
      <button
        type="button"
        class="inline-flex items-center justify-center w-5 h-5 text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors disabled:opacity-30"
        title={editor.maskOverlayVisible ? 'Hide mask overlays' : 'Show mask overlays'}
        aria-label="Toggle mask overlays"
        onclick={editor.toggleMaskOverlay}
      >
        <Icon path={editor.maskOverlayVisible ? mdiEye : mdiEyeOff} size={14} />
      </button>
      <div class="relative inline-flex items-center">
        <button
          bind:this={addLayerBtn}
          type="button"
          class="inline-flex items-center gap-1 h-5 px-1.5 rounded text-[11px] text-immich-dark-fg/70 hover:bg-white/10 hover:text-immich-dark-fg transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          title={cap.layersFull
            ? `You have used all ${N_MAX_MASK_LAYERS} masks. Delete one to add another.`
            : cap.totalFull
              ? `You have used all ${N_MAX_TOTAL_COMPONENTS} shapes across every mask. Delete one to add another.`
              : 'Create a new mask'}
          aria-label="New mask"
          disabled={cap.layersFull || cap.totalFull}
          onclick={toggleAddLayer}
        >
          <Icon path={mdiPlus} size={13} /> New
        </button>
        <MaskToolMenu
          open={addLayerOpen}
          anchor={addLayerBtn}
          align="right"
          target={{ layerId: null, mode: 'add' }}
          onClose={() => (addLayerOpen = false)}
        />
      </div>
    </div>
  </div>

  {#if editor.maskError}
    <div
      class="mx-1 px-2 py-1.5 rounded bg-red-500/10 border border-red-500/25 flex items-start gap-2"
      role="alert"
    >
      <div class="flex-1 text-[11px] text-red-200/90 leading-snug">
        Could not build that mask. {editor.maskError}
      </div>
      <div class="flex items-center gap-1 shrink-0">
        <button
          type="button"
          class="px-1.5 py-0.5 rounded text-[11px] text-red-100 bg-red-500/20 hover:bg-red-500/30 transition-colors disabled:opacity-40"
          disabled={editor.maskGenerating}
          onclick={() => void editor.retryMask()}
        >
          Try again
        </button>
        <button
          type="button"
          class="px-1.5 py-0.5 rounded text-[11px] text-red-100/70 hover:bg-red-500/20 transition-colors"
          onclick={editor.dismissMaskError}
        >
          Dismiss
        </button>
      </div>
    </div>
  {/if}

  {#if layers.length === 0}
    <div class="px-3 py-5 flex flex-col items-center gap-2 text-center">
      <div class="text-[11px] text-immich-dark-fg/50">
        Masks limit an adjustment to part of the photo.
      </div>
      <button
        type="button"
        class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-white/10 hover:bg-white/15 text-xs text-immich-dark-fg transition-colors"
        onclick={toggleAddLayer}
      >
        <Icon path={mdiPlus} size={13} /> Create a mask
      </button>
    </div>
  {:else}
    <div class="flex flex-col gap-0.5">
      {#each layers as layer, li (layer.id)}
        <MaskLayerRow {layer} index={li} total={layers.length} {cap} />
      {/each}
    </div>
  {/if}

  {#if layers.length > 0 && !active}
    <div class="px-1.5 pt-1 text-[11px] text-immich-dark-fg/40">Select a mask to adjust it.</div>
  {/if}

  {#if active}
    <div class="mt-2 border-t border-white/10 pt-2 flex flex-col gap-1.5">
      <div class="flex items-center gap-1 px-1">
        <button
          type="button"
          class="flex items-center gap-1 flex-1 min-w-0 text-left rounded hover:bg-white/5 transition-colors"
          aria-expanded={refineOpen}
          title={refineOpen
            ? 'Hide the shapes that build this mask'
            : 'Show the shapes that build this mask'}
          onclick={toggleRefine}
        >
          <Icon
            path={refineOpen ? mdiChevronDown : mdiChevronRight}
            size={14}
            class="opacity-40 shrink-0"
          />
          <span class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40 truncate">
            Refine ({active.components.length}/{N_MAX_COMPONENTS_PER_LAYER})
          </span>
        </button>
        <button
          type="button"
          class="shrink-0 px-1.5 py-0.5 rounded text-[11px] transition-colors disabled:opacity-30 disabled:cursor-not-allowed {active.invert
            ? 'bg-white/15 text-immich-dark-fg'
            : 'text-immich-dark-fg/60 hover:bg-white/10 hover:text-immich-dark-fg'}"
          aria-pressed={active.invert}
          title={active.invert
            ? 'Editing everything outside the shapes. Click to go back.'
            : 'Edit everything outside the shapes instead'}
          disabled={active.components.length === 0}
          onclick={() => void editor.toggleMaskLayerInvert(active.id)}
        >
          Invert
        </button>
      </div>

      {#if refineOpen}
        <div class="relative flex items-center gap-1 px-1">
          <button
            bind:this={addComponentBtn}
            type="button"
            class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] text-immich-dark-fg/70 hover:bg-white/10 hover:text-immich-dark-fg transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
            title="Add another shape to this mask"
            disabled={cap.componentsFull || cap.totalFull}
            onclick={() => openAddComponent('add')}
          >
            <Icon path={mdiPlus} size={12} /> Add
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] text-immich-dark-fg/70 hover:bg-white/10 hover:text-immich-dark-fg transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
            title="Cut a shape out of this mask"
            disabled={cap.componentsFull || cap.totalFull || active.components.length === 0}
            onclick={() => openAddComponent('subtract')}
          >
            <Icon path={mdiMinus} size={12} /> Subtract
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] text-immich-dark-fg/70 hover:bg-white/10 hover:text-immich-dark-fg transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
            title="Keep only where this shape overlaps the mask"
            disabled={cap.componentsFull || cap.totalFull || active.components.length === 0}
            onclick={() => openAddComponent('intersect')}
          >
            <Icon path={mdiSetCenter} size={12} /> Intersect
          </button>
          <MaskToolMenu
            open={addComponentOpen}
            anchor={addComponentBtn}
            align="left"
            heading
            target={{ layerId: active.id, mode: pendingMode }}
            onClose={() => (addComponentOpen = false)}
          />
        </div>

        {#if cap.componentsFull || cap.totalFull}
          <div class="px-1 text-[10px] text-immich-dark-fg/40">
            {cap.componentsFull
              ? `This mask is full at ${N_MAX_COMPONENTS_PER_LAYER} shapes. Delete one to add another.`
              : `You have used all ${N_MAX_TOTAL_COMPONENTS} shapes across every mask. Delete one to add another.`}
          </div>
        {/if}

        {#if active.components.length === 0}
          <div class="px-1 py-2 text-[11px] text-immich-dark-fg/30 italic">
            Empty mask. Use Add to pick a tool.
          </div>
        {:else}
          {#each active.components as comp, i (comp.id)}
            <MaskComponentRow
              layerId={active.id}
              {comp}
              index={i}
              total={active.components.length}
              label={compLabels[i]}
            />
          {/each}
        {/if}

        {#if activeComp && (activeComp.kind.kind === 'linear' || activeComp.kind.kind === 'radial')}
          <div class="mt-1 flex flex-col gap-2.5">
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
