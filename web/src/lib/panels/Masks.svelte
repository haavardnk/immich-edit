<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import MaskedEditSlider from './MaskedEditSlider.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import {
    N_MAX_MASK_LAYERS,
    type MaskComponent,
    type MaskComponentKind,
    type MaskComponentMode,
    type MaskLayer
  } from '$lib/types/edits';
  import { defaultLinear, defaultRadial } from '$lib/types/masks';
  import {
    mdiPlus,
    mdiClose,
    mdiEye,
    mdiEyeOff,
    mdiContentCopy,
    mdiGradientHorizontal,
    mdiCircleOutline,
    mdiBrush,
    mdiInvertColors,
    mdiCircleOpacity
  } from '@mdi/js';

  let addLayerOpen = $state(false);
  let addComponentOpen = $state(false);
  let addLayerBtn = $state<HTMLButtonElement | undefined>(undefined);
  let addLayerMenu = $state<HTMLDivElement | undefined>(undefined);
  let addComponentBtn = $state<HTMLButtonElement | undefined>(undefined);
  let addComponentMenu = $state<HTMLDivElement | undefined>(undefined);
  let addLayerPos = $state<{ top: number; right: number } | null>(null);
  let addComponentPos = $state<{ top: number; right: number } | null>(null);
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

  const amountValue = $derived(active?.amount ?? 1);
  const featherValue = $derived(
    activeComp && (activeComp.kind.kind === 'linear' || activeComp.kind.kind === 'radial')
      ? activeComp.kind.feather
      : 0.5
  );
  const brushSizeValue = $derived(editor.brushTool.size);
  const brushHardnessValue = $derived(editor.brushTool.hardness);
  const brushFlowValue = $derived(editor.brushTool.flow);

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
    return 'Brush';
  }

  function kindIcon(k: MaskComponentKind): string {
    if (k.kind === 'linear') return mdiGradientHorizontal;
    if (k.kind === 'radial') return mdiCircleOutline;
    return mdiBrush;
  }

  async function addLayer(kind: MaskComponentKind): Promise<void> {
    addLayerOpen = false;
    await editor.addMaskLayer(kind);
  }

  async function addBrushLayer(): Promise<void> {
    addLayerOpen = false;
    await editor.addBrushLayer();
  }

  async function addComponent(kind: MaskComponentKind): Promise<void> {
    if (!active) return;
    addComponentOpen = false;
    await editor.addMaskComponent(active.id, kind);
  }

  async function addBrushComp(): Promise<void> {
    if (!active) return;
    addComponentOpen = false;
    await editor.addBrushComponent(active.id);
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

  const MODES: { value: MaskComponentMode; label: string }[] = [
    { value: 'add', label: '+' },
    { value: 'subtract', label: '−' },
    { value: 'intersect', label: '∩' }
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

  function toggleAddComponent(): void {
    if (!addComponentOpen && addComponentBtn) {
      const r = addComponentBtn.getBoundingClientRect();
      addComponentPos = { top: r.bottom + 4, right: window.innerWidth - r.right };
    }
    addComponentOpen = !addComponentOpen;
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
      Layers ({layers.length}/{N_MAX_MASK_LAYERS})
    </div>
    <div class="flex items-center gap-1">
      <button
        type="button"
        class="inline-flex items-center justify-center w-5 h-5 text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors disabled:opacity-30"
        title={editor.maskOverlayVisible ? 'Hide overlays' : 'Show overlays'}
        aria-label="Toggle mask overlays"
        onclick={editor.toggleMaskOverlay}
      >
        <Icon path={editor.maskOverlayVisible ? mdiEye : mdiEyeOff} size={14} />
      </button>
      <div class="relative inline-flex items-center">
        <button
          bind:this={addLayerBtn}
          type="button"
          class="inline-flex items-center justify-center w-5 h-5 text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          title="Add mask layer"
          aria-label="Add mask layer"
          disabled={cap.layersFull || cap.totalFull}
          onclick={toggleAddLayer}
        >
          <Icon path={mdiPlus} size={14} />
        </button>
        {#if addLayerOpen && addLayerPos}
          <div
            bind:this={addLayerMenu}
            class="fixed z-50 bg-immich-dark-bg border border-white/10 rounded-md shadow-lg min-w-40"
            style="top: {addLayerPos.top}px; right: {addLayerPos.right}px"
          >
            <button
              type="button"
              class="flex items-center gap-2 w-full px-3 py-2 text-xs hover:bg-white/5 text-left"
              onclick={() => void addLayer(defaultLinear())}
            >
              <Icon path={mdiGradientHorizontal} size={14} /> Linear gradient
            </button>
            <button
              type="button"
              class="flex items-center gap-2 w-full px-3 py-2 text-xs hover:bg-white/5 text-left"
              onclick={() => void addLayer(defaultRadial())}
            >
              <Icon path={mdiCircleOutline} size={14} /> Radial gradient
            </button>
            <button
              type="button"
              class="flex items-center gap-2 w-full px-3 py-2 text-xs hover:bg-white/5 text-left"
              onclick={() => void addBrushLayer()}
            >
              <Icon path={mdiBrush} size={14} /> Brush
            </button>
          </div>
        {/if}
      </div>
    </div>
  </div>

  {#if layers.length === 0}
    <div class="px-1 py-4 text-[11px] text-immich-dark-fg/30 italic text-center">
      No mask layers. Use + to add one.
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
            title={isPreview ? 'Hide mask preview' : 'Show mask weight'}
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
        <div class="relative">
          <button
            bind:this={addComponentBtn}
            type="button"
            class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
            title="Add shape"
            aria-label="Add shape"
            disabled={cap.componentsFull || cap.totalFull}
            onclick={toggleAddComponent}
          >
            <Icon path={mdiPlus} size={14} />
          </button>
          {#if addComponentOpen && addComponentPos}
            <div
              bind:this={addComponentMenu}
              class="fixed z-50 bg-immich-dark-bg border border-white/10 rounded-md shadow-lg min-w-40"
              style="top: {addComponentPos.top}px; right: {addComponentPos.right}px"
            >
              <button
                type="button"
                class="flex items-center gap-2 w-full px-3 py-2 text-xs hover:bg-white/5 text-left"
                onclick={() => void addComponent(defaultLinear())}
              >
                <Icon path={mdiGradientHorizontal} size={14} /> Linear gradient
              </button>
              <button
                type="button"
                class="flex items-center gap-2 w-full px-3 py-2 text-xs hover:bg-white/5 text-left"
                onclick={() => void addComponent(defaultRadial())}
              >
                <Icon path={mdiCircleOutline} size={14} /> Radial gradient
              </button>
              <button
                type="button"
                class="flex items-center gap-2 w-full px-3 py-2 text-xs hover:bg-white/5 text-left"
                onclick={() => void addBrushComp()}
              >
                <Icon path={mdiBrush} size={14} /> Brush
              </button>
            </div>
          {/if}
        </div>
      </div>

      {#if active.components.length === 0}
        <div class="px-1 py-2 text-[11px] text-immich-dark-fg/30 italic">No shapes.</div>
      {:else}
        {#each active.components as comp, i (comp.id)}
          {@const isCompActive = editor.activeMaskComponentId === comp.id}
          <div
            class="flex items-center gap-1.5 px-1 py-0.5 rounded transition-colors cursor-pointer {isCompActive
              ? 'bg-white/10'
              : 'hover:bg-white/5'}"
            role="button"
            tabindex="0"
            onclick={() => editor.setActiveMaskComponent(comp.id)}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') editor.setActiveMaskComponent(comp.id);
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
            <Icon path={kindIcon(comp.kind)} size={12} class="opacity-50 shrink-0" />
            <span class="text-[11px] text-immich-dark-fg/70 truncate flex-1">
              {i + 1}. {kindLabel(comp.kind)}
            </span>
            {#if i > 0}
              <div class="flex rounded ring-1 ring-white/10 overflow-hidden text-[10px]">
                {#each MODES as m (m.value)}
                  <button
                    type="button"
                    class="px-1.5 leading-5 transition-colors {comp.mode === m.value
                      ? 'bg-white/15 text-immich-dark-fg'
                      : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
                    title={m.value}
                    onclick={(e) => {
                      e.stopPropagation();
                      setMode(active, comp, m.value);
                    }}
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
              onclick={(e) => {
                e.stopPropagation();
                toggleInvert(active, comp);
              }}
            >
              <Icon path={mdiInvertColors} size={12} />
            </button>
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
              onclick={(e) => e.stopPropagation()}
              class="w-14 slider-range"
              title="Opacity"
            />
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

    {#if activeComp && activeComp.kind.kind === 'brush'}
      <div class="mt-2 flex flex-col gap-2">
        <div class="flex items-center justify-between px-1">
          <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Brush</div>
          <div class="flex rounded ring-1 ring-white/10 overflow-hidden text-[10px]">
            <button
              type="button"
              class="px-2 leading-5 transition-colors {editor.brushTool.mode === 'paint'
                ? 'bg-white/15 text-immich-dark-fg'
                : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
              onclick={() => editor.setBrushTool({ mode: 'paint' })}
            >
              Paint
            </button>
            <button
              type="button"
              class="px-2 leading-5 transition-colors {editor.brushTool.mode === 'erase'
                ? 'bg-white/15 text-immich-dark-fg'
                : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
              onclick={() => editor.setBrushTool({ mode: 'erase' })}
            >
              Erase
            </button>
          </div>
        </div>
        <SliderRow
          label="Size"
          value={brushSizeValue}
          min={0.005}
          max={0.5}
          step={0.005}
          defaultValue={0.08}
          onLive={(v: number) => editor.setBrushTool({ size: v })}
          onCommit={() => editor.setBrushTool({ size: brushSizeValue })}
          format={(v: number) => v.toFixed(3)}
        />
        <SliderRow
          label="Hardness"
          value={brushHardnessValue}
          min={0}
          max={1}
          step={0.01}
          defaultValue={0.5}
          onLive={(v: number) => editor.setBrushTool({ hardness: v })}
          onCommit={() => editor.setBrushTool({ hardness: brushHardnessValue })}
          format={(v: number) => v.toFixed(2)}
        />
        <SliderRow
          label="Flow"
          value={brushFlowValue}
          min={0.01}
          max={1}
          step={0.01}
          defaultValue={0.8}
          onLive={(v: number) => editor.setBrushTool({ flow: v })}
          onCommit={() => editor.setBrushTool({ flow: brushFlowValue })}
          format={(v: number) => v.toFixed(2)}
        />
      </div>
    {/if}

    <div class="mt-3 border-t border-white/10 pt-3 flex flex-col gap-2.5">
      <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40 px-1">Adjustments</div>
      <MaskedEditSlider
        layerId={active.id}
        eKey="exposure_ev"
        label="Exposure"
        min={-5}
        max={5}
        step={0.05}
        format={(v: number) => v.toFixed(2)}
      />
      <MaskedEditSlider layerId={active.id} eKey="contrast" label="Contrast" min={-100} max={100} />
      <MaskedEditSlider
        layerId={active.id}
        eKey="highlights"
        label="Highlights"
        min={-100}
        max={100}
      />
      <MaskedEditSlider layerId={active.id} eKey="shadows" label="Shadows" min={-100} max={100} />
      <MaskedEditSlider layerId={active.id} eKey="whites" label="Whites" min={-100} max={100} />
      <MaskedEditSlider layerId={active.id} eKey="blacks" label="Blacks" min={-100} max={100} />
      <div class="border-t border-white/5"></div>
      <MaskedEditSlider
        layerId={active.id}
        eKey="saturation"
        label="Saturation"
        min={-100}
        max={100}
      />
      <MaskedEditSlider layerId={active.id} eKey="vibrance" label="Vibrance" min={-100} max={100} />
      <div class="border-t border-white/5"></div>
      <MaskedEditSlider
        layerId={active.id}
        eKey="wb_temp"
        label="Temperature"
        min={-100}
        max={100}
        gradient="linear-gradient(to right, #4a90d9, #b8a44c)"
      />
      <MaskedEditSlider
        layerId={active.id}
        eKey="wb_tint"
        label="Tint"
        min={-100}
        max={100}
        gradient="linear-gradient(to right, #b8508a, #6ab04c)"
      />
    </div>
  {/if}
</div>
