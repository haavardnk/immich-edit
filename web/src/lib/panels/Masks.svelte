<script lang="ts">
  import Notice from '#lib/components/Notice.svelte';
  import MaskAdjustments from './MaskAdjustments.svelte';
  import MaskClickTool from './MaskClickTool.svelte';
  import MaskShapeControls from './MaskShapeControls.svelte';
  import MaskComponentRow from './MaskComponentRow.svelte';
  import MaskLayerRow from './MaskLayerRow.svelte';
  import MaskToolMenu from './MaskToolMenu.svelte';
  import MaskOverlayMenu from './MaskOverlayMenu.svelte';
  import { ADD_MODES, addModeBlocked, clickPending } from './maskTools';
  import { applyReorder, useMaskPanel } from './maskPanel.svelte';
  import { shapeLabels } from './maskShapeText';
  import { Reorder } from '#lib/gestures/reorder.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import {
    N_MAX_MASK_LAYERS,
    N_MAX_COMPONENTS_PER_LAYER,
    N_MAX_TOTAL_COMPONENTS,
    type MaskComponent,
    type MaskComponentMode,
    type MaskLayer
  } from '#lib/types/edits.ts';
  import { Button, Icon, IconButton, Tooltip } from '@immich/ui';
  import { mergeProps } from 'bits-ui';
  import { mdiPlus, mdiEye, mdiEyeOff, mdiChevronDown, mdiChevronRight } from '@mdi/js';

  let layerMenu = $state<'header' | 'empty' | null>(null);
  let componentMenu = $state<MaskComponentMode | null>(null);

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
  const compLabels = $derived(shapeLabels(active));
  const refineOpen = $derived(
    active
      ? (editor.maskRefineOpen[active.id] ??
          (active.components.length === 0 || activeComp !== null))
      : false
  );
  const pendingClick = $derived(clickPending());
  const layerOrder = $derived(layers.map((layer) => layer.id));
  const shapeOrder = $derived(active ? active.components.map((comp) => comp.id) : []);
  const reorder = new Reorder(applyReorder);

  useMaskPanel();

  function toggleRefine(): void {
    if (!active) return;
    editor.setMaskRefineOpen(active.id, !refineOpen);
  }

  function openAddComponent(mode: MaskComponentMode, open: boolean): void {
    if (open && active) editor.setMaskRefineOpen(active.id, true);
    componentMenu = open ? mode : null;
  }
</script>

<div class="flex flex-col gap-1">
  <div class="flex h-8 items-center justify-between border-b border-hairline px-1">
    <div class="text-[9px] font-semibold uppercase text-dark/65">
      Masks ({layers.length}/{N_MAX_MASK_LAYERS})
    </div>
    <div class="flex items-center gap-1">
      <div class="flex items-center">
        <IconButton
          size="small"
          variant="ghost"
          color="secondary"
          class="bg-transparent hover:bg-light-950/6"
          icon={editor.maskOverlayVisible ? mdiEye : mdiEyeOff}
          title={hint(
            editor.maskOverlayVisible ? 'Hide mask overlays' : 'Show mask overlays',
            'maskOverlay'
          )}
          aria-label="Toggle mask overlays"
          aria-pressed={editor.maskOverlayVisible}
          onclick={editor.toggleMaskOverlay}
        />
        <MaskOverlayMenu />
      </div>
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
                  class="h-8 bg-transparent hover:bg-light-950/6"
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
    <div role="list" aria-label="Masks" class="flex flex-col gap-0.5">
      {#each layers as layer, li (layer.id)}
        <MaskLayerRow {layer} index={li} order={layerOrder} {reorder} {cap} />
      {/each}
    </div>
  {/if}

  {#if pendingClick && editor.clickTool.layerId === null}
    <div class="px-1.5">
      <MaskClickTool layerId={null} pending />
    </div>
  {:else if layers.length > 0 && !active}
    <div class="px-1.5 pt-1 text-[11px] text-dark/65">Select a mask to adjust it.</div>
  {/if}

  {#if active}
    <div class="mt-1 flex flex-col gap-1 rounded-md border border-hairline bg-light-950/3 p-1">
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
          {#each ADD_MODES as menu (menu.mode)}
            <MaskToolMenu
              open={componentMenu === menu.mode}
              align="start"
              heading
              target={{ layerId: active.id, mode: menu.mode }}
              onOpenChange={(v) => openAddComponent(menu.mode, v)}
            >
              {#snippet trigger(menuProps)}
                <Tooltip text={menu.tip}>
                  {#snippet child({ props })}
                    <Button
                      type="button"
                      size="tiny"
                      variant="ghost"
                      color="secondary"
                      leadingIcon={menu.icon}
                      disabled={addModeBlocked(menu.mode, active.id, active.components.length)}
                      {...mergeProps(props, menuProps)}
                    >
                      {menu.label}
                    </Button>
                  {/snippet}
                </Tooltip>
              {/snippet}
            </MaskToolMenu>
          {/each}
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
          <div role="list" aria-label="Shapes" class="flex flex-col gap-1">
            {#each active.components as comp, i (comp.id)}
              <MaskComponentRow
                layerId={active.id}
                {comp}
                index={i}
                order={shapeOrder}
                {reorder}
                label={compLabels[i] ?? ''}
              />
            {/each}
          </div>
        {/if}

        {#if activeComp}
          <MaskShapeControls layerId={active.id} component={activeComp} />
        {:else if pendingClick && editor.clickTool.layerId === active.id}
          <MaskClickTool layerId={active.id} pending />
        {/if}
      {/if}
    </div>

    <MaskAdjustments layerId={active.id} />
  {/if}
</div>
