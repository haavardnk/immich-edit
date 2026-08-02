<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import MaskAdjustments from './MaskAdjustments.svelte';
  import MaskBrushControls from './MaskBrushControls.svelte';
  import MaskGeneratedControls from './MaskGeneratedControls.svelte';
  import MaskRangeControls from './MaskRangeControls.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import {
    N_MAX_MASK_LAYERS,
    type MaskComponent,
    type MaskComponentKind,
    type MaskComponentMode,
    type MaskLayer
  } from '$lib/types/edits';
  import {
    defaultColorRange,
    defaultLinear,
    defaultLumaRange,
    defaultRadial,
    type ManualTool
  } from '$lib/types/masks';
  import {
    mdiPlus,
    mdiMinus,
    mdiSetCenter,
    mdiClose,
    mdiEye,
    mdiEyeOff,
    mdiContentCopy,
    mdiGradientHorizontal,
    mdiCircleOutline,
    mdiBrush,
    mdiBrightness6,
    mdiInvertColors,
    mdiPalette,
    mdiCircleOpacity,
    mdiAutoFix
  } from '@mdi/js';
  import MaskToolPicker from '$lib/components/editor/MaskToolPicker.svelte';
  import { listMaskModels, type MaskKind, type SemanticClass } from '$lib/api/masks';
  import { session } from '$lib/stores/session.svelte';
  import { toasts } from '$lib/stores/toasts.svelte';
  import { goto } from '$app/navigation';

  let maskKinds = $state<{ kind: MaskKind; installed: boolean }[]>([]);
  let semanticClasses = $state<SemanticClass[]>([]);
  let segmentEnabled = $state(false);
  let modelsRequested = $state(false);
  const clickInstalled = $derived(
    maskKinds.some((entry) => entry.kind === 'click' && entry.installed)
  );

  let addLayerOpen = $state(false);
  let addComponentOpen = $state(false);
  let pendingMode = $state<MaskComponentMode>('add');
  let addLayerBtn = $state<HTMLButtonElement | undefined>(undefined);
  let addLayerMenu = $state<HTMLDivElement | undefined>(undefined);
  let addComponentBtn = $state<HTMLButtonElement | undefined>(undefined);
  let addComponentMenu = $state<HTMLDivElement | undefined>(undefined);
  let addLayerPos = $state<{ top: number; right: number } | null>(null);
  let addComponentPos = $state<{ top: number; left: number } | null>(null);
  let editingNameId = $state<string | null>(null);
  let nameDraft = $state('');
  const layers = $derived(editor.edits.masks);
  const active = $derived<MaskLayer | null>(
    editor.activeLayerId ? layers.find((l) => l.id === editor.activeLayerId) ?? null : null
  );
  const activeComp = $derived<MaskComponent | null>(
    active && editor.activeMaskComponentId
      ? active.components.find((c) => c.id === editor.activeMaskComponentId) ?? null
      : null
  );
  const cap = $derived(editor.maskCapacityFor(editor.activeLayerId));
  const isPreviewingActive = $derived(active !== null && editor.maskPreviewLayerId === active.id);
  const refineActive = $derived(
    editor.clickTool.active && active !== null && editor.clickTool.layerId === active.id
  );

  const amountValue = $derived(active?.amount ?? 1);
  const featherValue = $derived(
    activeComp && (activeComp.kind.kind === 'linear' || activeComp.kind.kind === 'radial')
      ? activeComp.kind.feather
      : 0.5
  );

  $effect(() => {
    if (modelsRequested) return;
    modelsRequested = true;
    listMaskModels()
      .then((m) => {
        segmentEnabled = m.enabled;
        semanticClasses = m.semantic_classes ?? [];
        maskKinds = m.enabled
          ? [...new Set(m.models.map((x) => x.kind))].map((kind) => ({
              kind,
              installed: m.models.some((x) => x.kind === kind && x.installed)
            }))
          : [];
      })
      .catch(() => {
        segmentEnabled = false;
        maskKinds = [];
        semanticClasses = [];
      });
  });

  const lumaMinValue = $derived(
    activeComp?.kind.kind === 'luma_range' ? activeComp.kind.min : 0.25
  );

  function onAmountLive(v: number): void {
    if (active) editor.setMaskLayerAmount(active.id, v);
  }

  function onAmountCommit(): void {
    void editor.commitMasks();
  }

  function onFeatherLive(v: number): void {
    if (active && activeComp) editor.setMaskComponentFeather(active.id, activeComp.id, v);
  }

  function onFeatherCommit(): void {
    void editor.commitMasks();
  }

  function kindLabel(k: MaskComponentKind): string {
    if (k.kind === 'linear') return 'Linear gradient';
    if (k.kind === 'radial') return 'Radial gradient';
    if (k.kind === 'brush') return 'Brush';
    if (k.kind === 'luma_range') return 'Luminance range';
    return 'Color range';
  }

  function kindIcon(k: MaskComponentKind): string {
    if (k.kind === 'linear') return mdiGradientHorizontal;
    if (k.kind === 'radial') return mdiCircleOutline;
    if (k.kind === 'brush') return mdiBrush;
    if (k.kind === 'luma_range') return mdiBrightness6;
    return mdiPalette;
  }

  async function addLayer(kind: MaskComponentKind): Promise<void> {
    addLayerOpen = false;
    await editor.addMaskLayer(kind);
  }

  async function addBrushLayer(): Promise<void> {
    addLayerOpen = false;
    await editor.addBrushLayer();
  }

  function manualKind(tool: ManualTool): MaskComponentKind | null {
    if (tool === 'linear') return defaultLinear();
    if (tool === 'radial') return defaultRadial();
    if (tool === 'luma_range') return defaultLumaRange();
    if (tool === 'color_range') return defaultColorRange();
    return null;
  }

  async function pickLayerManual(tool: ManualTool): Promise<void> {
    const kind = manualKind(tool);
    if (kind) await addLayer(kind);
    else await addBrushLayer();
  }

  async function pickLayerAi(kind: MaskKind, installed: boolean, maskClass?: string): Promise<void> {
    if (!installed) {
      promptInstall(kind);
      return;
    }
    await addGenerated(kind, maskClass);
  }

  async function pickShapeManual(tool: ManualTool): Promise<void> {
    const kind = manualKind(tool);
    if (kind) await addComponent(kind);
    else await addBrushComp();
  }

  async function pickShapeAi(kind: MaskKind, installed: boolean, maskClass?: string): Promise<void> {
    if (!installed) {
      addComponentOpen = false;
      promptInstall(kind);
      return;
    }
    if (kind === 'click') {
      armClickComponent();
      return;
    }
    await addGeneratedComp(kind, maskClass);
  }

  async function addGenerated(kind: MaskKind, maskClass?: string): Promise<void> {
    addLayerOpen = false;
    if (kind === 'click') {
      editor.setActiveMaskComponent(null);
      editor.clickTool = { active: true, negative: false, box: false, layerId: null, mode: 'add' };
      toasts.push('info', 'Click the photo to build a mask. Shift-click removes areas.');
      return;
    }
    await editor.addGeneratedLayer(kind, maskClass);
  }

  function promptInstall(kind: MaskKind): void {
    addLayerOpen = false;
    if (session.isAdmin) {
      void goto('/settings');
      return;
    }
    toasts.push(
      'info',
      `${generatedLabel(kind)} masks need a model. Ask an administrator to download one in Settings.`
    );
  }

  function generatedLabel(kind: string): string {
    if (kind === 'subject') return 'Subject';
    if (kind === 'people') return 'People';
    if (kind === 'sky') return 'Sky';
    if (kind === 'depth') return 'Depth';
    if (kind === 'semantic') return 'Scene';
    if (kind === 'click') return 'Click to select';
    return kind;
  }

  async function addComponent(kind: MaskComponentKind): Promise<void> {
    if (!active) return;
    addComponentOpen = false;
    await editor.addMaskComponent(active.id, kind, pendingMode);
  }

  async function addBrushComp(): Promise<void> {
    if (!active) return;
    addComponentOpen = false;
    await editor.addBrushComponent(active.id, pendingMode);
  }

  function armClickComponent(): void {
    if (!active) return;
    addComponentOpen = false;
    if (!clickInstalled) {
      promptInstall('click');
      return;
    }
    editor.setActiveMaskComponent(null);
    editor.clickTool = {
      active: true,
      negative: false,
      box: false,
      layerId: active.id,
      mode: pendingMode
    };
  }

  function armLayerBox(): void {
    addLayerOpen = false;
    editor.setActiveMaskComponent(null);
    editor.clickTool = { active: true, negative: false, box: true, layerId: null, mode: 'add' };
    toasts.push('info', 'Drag a box around the subject.');
  }

  function armShapeBox(): void {
    if (!active) return;
    addComponentOpen = false;
    editor.setActiveMaskComponent(null);
    editor.clickTool = {
      active: true,
      negative: false,
      box: true,
      layerId: active.id,
      mode: pendingMode
    };
    toasts.push('info', 'Drag a box around the subject.');
  }

  async function addGeneratedComp(kind: MaskKind, maskClass?: string): Promise<void> {
    if (!active) return;
    addComponentOpen = false;
    await editor.addGeneratedComponent(active.id, kind, pendingMode, maskClass);
  }

  function beginRename(layer: MaskLayer): void {
    editingNameId = layer.id;
    nameDraft = layer.name;
  }

  async function commitRename(layer: MaskLayer): Promise<void> {
    const next = nameDraft.trim();
    editingNameId = null;
    if (next && next !== layer.name) {
      await editor.renameMaskLayer(layer.id, next);
    }
  }

  function setMode(layer: MaskLayer, comp: MaskComponent, mode: MaskComponentMode): void {
    editor.patchMaskComponent(layer.id, comp.id, { mode }, false);
    void editor.commitMasks();
  }

  function toggleComp(layer: MaskLayer, comp: MaskComponent): void {
    editor.patchMaskComponent(layer.id, comp.id, { enabled: !comp.enabled }, false);
    void editor.commitMasks();
  }

  function toggleInvert(layer: MaskLayer, comp: MaskComponent): void {
    editor.patchMaskComponent(layer.id, comp.id, { invert: !comp.invert }, false);
    void editor.commitMasks();
  }

  function setCompOpacity(layer: MaskLayer, comp: MaskComponent, opacity: number): void {
    editor.patchMaskComponent(layer.id, comp.id, { opacity }, true);
  }

  function commitComp(): void {
    void editor.commitMasks();
  }

  function togglePreview(layer: MaskLayer): void {
    if (editor.maskPreviewLayerId === layer.id) editor.endMaskPreview();
    else editor.previewMaskWeight(layer.id);
  }

  const MODES: { value: MaskComponentMode; label: string; hint: string }[] = [
    { value: 'add', label: '+', hint: 'Add this shape to the mask' },
    { value: 'subtract', label: '−', hint: 'Cut this shape out of the mask' },
    { value: 'intersect', label: '∩', hint: 'Keep only where this shape overlaps' }
  ];

  function focusOnMount(node: HTMLInputElement): void {
    node.focus();
    node.select();
  }

  function toggleAddLayer(): void {
    if (!addLayerOpen && addLayerBtn) {
      const r = addLayerBtn.getBoundingClientRect();
      addLayerPos = { top: r.bottom + 4, right: window.innerWidth - r.right };
    }
    addLayerOpen = !addLayerOpen;
  }

  function openAddComponent(mode: MaskComponentMode): void {
    pendingMode = mode;
    if (!addComponentOpen && addComponentBtn) {
      const r = addComponentBtn.getBoundingClientRect();
      addComponentPos = { top: r.bottom + 4, left: r.left };
    }
    addComponentOpen = !addComponentOpen;
  }

  function modeVerb(mode: MaskComponentMode): string {
    if (mode === 'subtract') return 'Subtract';
    if (mode === 'intersect') return 'Intersect';
    return 'Add';
  }

  function setRefine(negative: boolean): void {
    if (!active) return;
    editor.clickTool = { active: true, negative, box: false, layerId: active.id, mode: 'add' };
  }

  function stopRefine(): void {
    editor.clickTool = { active: false, negative: false, box: false, layerId: null, mode: 'add' };
  }

  $effect(() => {
    if (!addLayerOpen && !addComponentOpen) return;
    function onDown(e: PointerEvent): void {
      const t = e.target as Node;
      if (
        addLayerOpen &&
        !(addLayerBtn?.contains(t) ?? false) &&
        !(addLayerMenu?.contains(t) ?? false)
      )
        addLayerOpen = false;
      if (
        addComponentOpen &&
        !(addComponentBtn?.contains(t) ?? false) &&
        !(addComponentMenu?.contains(t) ?? false)
      )
        addComponentOpen = false;
    }
    function onScrollOrResize(): void {
      addLayerOpen = false;
      addComponentOpen = false;
    }
    window.addEventListener('pointerdown', onDown, true);
    window.addEventListener('resize', onScrollOrResize);
    window.addEventListener('scroll', onScrollOrResize, true);
    return () => {
      window.removeEventListener('pointerdown', onDown, true);
      window.removeEventListener('resize', onScrollOrResize);
      window.removeEventListener('scroll', onScrollOrResize, true);
    };
  });
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
          title="Create a new mask"
          aria-label="New mask"
          disabled={cap.layersFull || cap.totalFull}
          onclick={toggleAddLayer}
        >
          <Icon path={mdiPlus} size={13} /> New
        </button>
        {#if addLayerOpen && addLayerPos}
          <div
            bind:this={addLayerMenu}
            class="fixed z-50 bg-immich-dark-gray border border-white/10 rounded-lg shadow-xl"
            style="top: {addLayerPos.top}px; right: {addLayerPos.right}px"
          >
            <MaskToolPicker
              aiKinds={maskKinds}
              semanticClasses={semanticClasses}
              busy={editor.maskGenerating}
              onManual={(tool) => void pickLayerManual(tool)}
              onAi={(kind, installed, cls) => void pickLayerAi(kind, installed, cls)}
              onBox={armLayerBox}
            />
          </div>
        {/if}
      </div>
    </div>
  </div>

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
      {#each layers as layer (layer.id)}
        {@const isActive = editor.activeLayerId === layer.id}
        {@const isPreview = editor.maskPreviewLayerId === layer.id}
        <div
          class="flex items-center gap-1.5 px-1.5 py-1 rounded transition-colors cursor-pointer {isActive
            ? 'bg-white/10'
            : 'hover:bg-white/5'}"
          role="button"
          tabindex="0"
          onclick={() => editor.setActiveLayer(layer.id)}
          onkeydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') editor.setActiveLayer(layer.id);
          }}
        >
          <button
            type="button"
            class="w-3 h-3 rounded-sm ring-1 ring-white/20 shrink-0"
            style="background-color: {layer.color}"
            title="Layer color"
            aria-label="Layer color"
            onclick={(e) => {
              e.stopPropagation();
            }}
          ></button>
          <button
            type="button"
            class="shrink-0 text-immich-dark-fg/50 hover:text-immich-dark-fg"
            title={layer.enabled ? 'Disable layer' : 'Enable layer'}
            aria-label="Toggle layer"
            onclick={(e) => {
              e.stopPropagation();
              void editor.toggleMaskLayerEnabled(layer.id);
            }}
          >
            <Icon path={layer.enabled ? mdiEye : mdiEyeOff} size={13} />
          </button>
          {#if editingNameId === layer.id}
            <input
              class="flex-1 bg-white/5 border border-white/10 rounded px-1 text-xs text-immich-dark-fg outline-none"
              bind:value={nameDraft}
              onblur={() => void commitRename(layer)}
              onkeydown={(e) => {
                if (e.key === 'Enter') (e.currentTarget as HTMLInputElement).blur();
                else if (e.key === 'Escape') {
                  editingNameId = null;
                }
              }}
              use:focusOnMount
              onclick={(e) => e.stopPropagation()}
            />
          {:else}
            <button
              type="button"
              class="flex-1 text-left text-xs text-immich-dark-fg/90 truncate {layer.enabled
                ? ''
                : 'opacity-50'}"
              ondblclick={(e) => {
                e.stopPropagation();
                beginRename(layer);
              }}
              onclick={(e) => {
                e.stopPropagation();
                editor.setActiveLayer(layer.id);
              }}
              title="Double-click to rename"
            >
              {layer.name}
            </button>
          {/if}
          <button
            type="button"
            class="shrink-0 text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors {isPreview
              ? 'text-immich-dark-primary'
              : ''}"
            title={isPreview ? 'Hide mask overlay' : 'Show mask overlay'}
            aria-label="Toggle mask preview"
            onclick={(e) => {
              e.stopPropagation();
              togglePreview(layer);
            }}
          >
            <Icon path={mdiCircleOpacity} size={13} />
          </button>
          <button
            type="button"
            class="shrink-0 text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
            title="Duplicate layer"
            aria-label="Duplicate layer"
            disabled={cap.layersFull || cap.totalFull}
            onclick={(e) => {
              e.stopPropagation();
              void editor.duplicateMaskLayer(layer.id);
            }}
          >
            <Icon path={mdiContentCopy} size={12} />
          </button>
          <button
            type="button"
            class="shrink-0 text-immich-dark-fg/40 hover:text-red-400 transition-colors"
            title="Delete layer"
            aria-label="Delete layer"
            onclick={(e) => {
              e.stopPropagation();
              void editor.removeMaskLayer(layer.id);
            }}
          >
            <Icon path={mdiClose} size={13} />
          </button>
        </div>
      {/each}
    </div>
  {/if}

  {#if active}
    <div class="mt-2 border-t border-white/10 pt-3 flex flex-col gap-2.5">
      <SliderRow
        label="Amount"
        value={amountValue}
        min={0}
        max={1}
        step={0.01}
        defaultValue={1}
        onLive={onAmountLive}
        onCommit={onAmountCommit}
        format={(v: number) => v.toFixed(2)}
      />
    </div>

    <div class="mt-3 flex flex-col gap-1.5">
      <div class="flex items-center justify-between px-1">
        <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">
          Shapes ({active.components.length})
        </div>
        <button
          type="button"
          class="inline-flex items-center gap-1 h-5 px-1.5 rounded text-[10px] transition-colors {isPreviewingActive
            ? 'bg-white/15 text-immich-dark-fg'
            : 'text-immich-dark-fg/50 hover:bg-white/10 hover:text-immich-dark-fg'}"
          title="Show the finished mask over the photo"
          onclick={() => togglePreview(active)}
        >
          <Icon path={mdiCircleOpacity} size={12} /> Show mask
        </button>
      </div>

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
        {#if addComponentOpen && addComponentPos}
          <div
            bind:this={addComponentMenu}
            class="fixed z-50 bg-immich-dark-gray border border-white/10 rounded-lg shadow-xl"
            style="top: {addComponentPos.top}px; left: {addComponentPos.left}px"
          >
            <div
              class="px-3 pt-2 text-[10px] uppercase tracking-wider text-immich-dark-fg/40 border-b border-white/10 pb-2"
            >
              {modeVerb(pendingMode)} with
            </div>
            <MaskToolPicker
              aiKinds={maskKinds}
              semanticClasses={semanticClasses}
              busy={editor.maskGenerating}
              onManual={(tool) => void pickShapeManual(tool)}
              onAi={(kind, installed, cls) => void pickShapeAi(kind, installed, cls)}
              onBox={armShapeBox}
            />
          </div>
        {/if}
      </div>

      {#if active.components.length === 0}
        <div class="px-1 py-2 text-[11px] text-immich-dark-fg/30 italic">
          Empty mask. Use Add to pick a tool.
        </div>
      {:else}
        {#each active.components as comp, i (comp.id)}
          {@const isCompActive = editor.activeMaskComponentId === comp.id}
          <div
            class="flex items-center gap-1.5 px-1 py-0.5 rounded transition-colors cursor-pointer {isCompActive
              ? 'bg-white/10'
              : 'hover:bg-white/5'}"
            role="button"
            tabindex="0"
            onclick={() => editor.setActiveMaskComponent(isCompActive ? null : comp.id)}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ')
                editor.setActiveMaskComponent(isCompActive ? null : comp.id);
            }}
          >
            <button
              type="button"
              class="shrink-0 text-immich-dark-fg/50 hover:text-immich-dark-fg"
              title={comp.enabled ? 'Disable shape' : 'Enable shape'}
              aria-label="Toggle shape"
              onclick={(e) => {
                e.stopPropagation();
                toggleComp(active, comp);
              }}
            >
              <Icon path={comp.enabled ? mdiEye : mdiEyeOff} size={12} />
            </button>
            <Icon
              path={comp.generated ? mdiAutoFix : kindIcon(comp.kind)}
              size={12}
              class="opacity-50 shrink-0"
            />
            <span class="text-[11px] text-immich-dark-fg/70 truncate flex-1">
              {comp.generated ? generatedLabel(comp.generated.kind) : kindLabel(comp.kind)}
            </span>
            {#if i === 0}
              <span
                class="shrink-0 text-[9px] uppercase tracking-wider text-immich-dark-fg/30"
                title="The first shape starts the mask">Base</span
              >
            {/if}
            <button
              type="button"
              class="shrink-0 text-immich-dark-fg/40 hover:text-red-400 transition-colors"
              title="Delete shape"
              aria-label="Delete shape"
              onclick={(e) => {
                e.stopPropagation();
                void editor.removeMaskComponent(active.id, comp.id);
              }}
            >
              <Icon path={mdiClose} size={12} />
            </button>
          </div>
          {#if isCompActive}
            <div class="flex items-center gap-2 px-2 pb-1.5 pt-1">
              {#if i > 0}
                <div class="flex rounded ring-1 ring-white/10 overflow-hidden text-[10px]">
                  {#each MODES as m (m.value)}
                    <button
                      type="button"
                      class="px-1.5 leading-5 transition-colors {comp.mode === m.value
                        ? 'bg-white/15 text-immich-dark-fg'
                        : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
                      title={m.hint}
                      onclick={() => setMode(active, comp, m.value)}
                    >
                      {m.label}
                    </button>
                  {/each}
                </div>
              {/if}
              <button
                type="button"
                class="shrink-0 text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors {comp.invert
                  ? 'text-immich-dark-primary'
                  : ''}"
                title={comp.invert ? 'Invert on' : 'Invert off'}
                aria-label="Invert shape"
                onclick={() => toggleInvert(active, comp)}
              >
                <Icon path={mdiInvertColors} size={12} />
              </button>
              <span class="text-[10px] text-immich-dark-fg/40">Opacity</span>
              <input
                type="range"
                min={0}
                max={1}
                step={0.01}
                value={comp.opacity}
                oninput={(e) =>
                  setCompOpacity(
                    active,
                    comp,
                    parseFloat((e.currentTarget as HTMLInputElement).value)
                  )}
                onpointerup={commitComp}
                onkeyup={commitComp}
                class="flex-1 slider-range"
                title="Opacity"
              />
            </div>
          {/if}
        {/each}
      {/if}
    </div>

    {#if activeComp && (activeComp.kind.kind === 'linear' || activeComp.kind.kind === 'radial')}
      <div class="mt-2 flex flex-col gap-2.5">
        <SliderRow
          label="Feather"
          value={featherValue}
          min={0}
          max={1}
          step={0.01}
          defaultValue={0.5}
          onLive={onFeatherLive}
          onCommit={onFeatherCommit}
          format={(v: number) => v.toFixed(2)}
        />
      </div>
    {/if}

    {#if activeComp && activeComp.kind.kind === 'brush' && clickInstalled}
      <div class="mt-2 flex items-center justify-between gap-2 px-1">
        <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Click to refine</div>
        <div class="flex items-center gap-2">
          <div class="flex rounded ring-1 ring-white/10 overflow-hidden text-[10px]">
            <button
              type="button"
              class="px-2 leading-5 transition-colors {refineActive && !editor.clickTool.negative
                ? 'bg-white/15 text-immich-dark-fg'
                : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
              title="Click the photo to add that area to this shape"
              onclick={() => setRefine(false)}>Add</button
            >
            <button
              type="button"
              class="px-2 leading-5 transition-colors {refineActive && editor.clickTool.negative
                ? 'bg-white/15 text-immich-dark-fg'
                : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
              title="Click the photo to cut that area out of this shape"
              onclick={() => setRefine(true)}>Remove</button
            >
          </div>
          {#if refineActive}
            <button
              type="button"
              class="text-[10px] text-immich-dark-fg/50 hover:text-immich-dark-fg"
              onclick={stopRefine}>Done</button
            >
          {/if}
        </div>
      </div>
      {#if editor.maskGenerating}
        <div class="px-1 text-[10px] text-immich-dark-fg/40">Working…</div>
      {/if}
    {/if}

    {#if activeComp?.generated}
      <MaskGeneratedControls layerId={active.id} component={activeComp} />
    {:else if activeComp && activeComp.kind.kind === 'brush'}
      <MaskBrushControls />
    {/if}

    {#if activeComp && (activeComp.kind.kind === 'luma_range' || activeComp.kind.kind === 'color_range')}
      <MaskRangeControls layerId={active.id} component={activeComp} />
    {/if}

    <MaskAdjustments layerId={active.id} />
  {/if}
</div>
