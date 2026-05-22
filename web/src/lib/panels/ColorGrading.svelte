<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import HueWheel from '$lib/components/editor/controls/HueWheel.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiRestore, mdiChevronDown, mdiChevronRight } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';

  type RegionKey = 'shadows' | 'midtones' | 'highlights' | 'global';
  type Mode = 'three_way' | 'global';

  const REGION_LABELS: Record<RegionKey, string> = {
    shadows: 'Shadows',
    midtones: 'Midtones',
    highlights: 'Highlights',
    global: 'Global'
  };

  const ALL_REGIONS: RegionKey[] = ['shadows', 'midtones', 'highlights', 'global'];

  let mode = $state<Mode>('three_way');
  let activeRegion = $state<RegionKey>('midtones');
  let adjustOpen = $state(false);

  $effect(() => {
    if (mode === 'global') activeRegion = 'global';
    else if (activeRegion === 'global') activeRegion = 'midtones';
  });

  const activeRegionData = $derived(editor.edits.color.color_grade[activeRegion]);
  const hueGradient =
    'linear-gradient(to right, hsl(0,100%,50%), hsl(30,100%,50%), hsl(60,100%,50%), hsl(90,100%,50%), hsl(120,100%,50%), hsl(150,100%,50%), hsl(180,100%,50%), hsl(210,100%,50%), hsl(240,100%,50%), hsl(270,100%,50%), hsl(300,100%,50%), hsl(330,100%,50%), hsl(360,100%,50%))';
  const satGradient = $derived(
    `linear-gradient(to right, hsl(${Math.round(activeRegionData.hue)}, 0%, 50%), hsl(${Math.round(activeRegionData.hue)}, 100%, 50%))`
  );
  const lumGradient = $derived(
    `linear-gradient(to right, hsl(${Math.round(activeRegionData.hue)}, ${Math.round(activeRegionData.sat)}%, 0%), hsl(${Math.round(activeRegionData.hue)}, ${Math.round(activeRegionData.sat)}%, 50%), hsl(${Math.round(activeRegionData.hue)}, ${Math.round(activeRegionData.sat)}%, 100%))`
  );
  const balanceGradient = 'linear-gradient(to right, #2a6cff, #555, #ffae42)';
  const blendingGradient = 'linear-gradient(to right, #222, #888)';

  function resetRegion(key: RegionKey): void {
    const reg = editor.edits.color.color_grade[key];
    reg.hue = 0;
    reg.sat = 0;
    reg.lum = 0;
    editor.onCommit();
  }

  function resetAllGrading(): void {
    const cg = editor.edits.color.color_grade;
    for (const k of ALL_REGIONS) {
      cg[k].hue = 0;
      cg[k].sat = 0;
      cg[k].lum = 0;
    }
    cg.balance = 0;
    cg.blend = 0;
    editor.onCommit();
  }
</script>

<div class="flex flex-col gap-2.5">
  <div class="flex items-center justify-between px-1">
    <div class="flex rounded ring-1 ring-white/10 overflow-hidden text-[10px] uppercase tracking-wide">
      <button
        type="button"
        class="px-2 py-1 transition-colors {mode === 'three_way' ? 'bg-white/15 text-immich-dark-fg' : 'text-immich-dark-fg/60 hover:text-immich-dark-fg'}"
        onclick={() => (mode = 'three_way')}
      >
        3-Way
      </button>
      <button
        type="button"
        class="px-2 py-1 transition-colors {mode === 'global' ? 'bg-white/15 text-immich-dark-fg' : 'text-immich-dark-fg/60 hover:text-immich-dark-fg'}"
        onclick={() => (mode = 'global')}
      >
        Global
      </button>
    </div>
    <button
      type="button"
      class="flex items-center gap-1 text-[10px] text-immich-dark-fg/60 hover:text-immich-dark-fg transition-colors"
      title="Reset all color grading"
      onclick={resetAllGrading}
    >
      <Icon path={mdiRestore} size={12} />
      Reset
    </button>
  </div>

  {#if mode === 'three_way'}
    <div class="cg-triangle">
      <div class="cg-cell cg-cell-top">
        <button
          type="button"
          class="cg-wheel-btn {activeRegion === 'midtones' ? 'is-active' : ''}"
          onclick={() => (activeRegion = 'midtones')}
          title="Midtones"
        >
          <HueWheel
            bind:hue={editor.edits.color.color_grade.midtones.hue}
            bind:sat={editor.edits.color.color_grade.midtones.sat}
            size={108}
            onLive={editor.onLive}
            onCommit={editor.onCommit}
          />
        </button>
        <button
          type="button"
          class="cg-label cg-label-btn {activeRegion === 'midtones' ? 'is-active' : ''}"
          onclick={() => (activeRegion = 'midtones')}
        >
          Midtones
        </button>
      </div>
      <div class="cg-cell">
        <button
          type="button"
          class="cg-wheel-btn {activeRegion === 'shadows' ? 'is-active' : ''}"
          onclick={() => (activeRegion = 'shadows')}
          title="Shadows"
        >
          <HueWheel
            bind:hue={editor.edits.color.color_grade.shadows.hue}
            bind:sat={editor.edits.color.color_grade.shadows.sat}
            size={92}
            onLive={editor.onLive}
            onCommit={editor.onCommit}
          />
        </button>
        <button
          type="button"
          class="cg-label cg-label-btn {activeRegion === 'shadows' ? 'is-active' : ''}"
          onclick={() => (activeRegion = 'shadows')}
        >
          Shadows
        </button>
      </div>
      <div class="cg-cell">
        <button
          type="button"
          class="cg-wheel-btn {activeRegion === 'highlights' ? 'is-active' : ''}"
          onclick={() => (activeRegion = 'highlights')}
          title="Highlights"
        >
          <HueWheel
            bind:hue={editor.edits.color.color_grade.highlights.hue}
            bind:sat={editor.edits.color.color_grade.highlights.sat}
            size={92}
            onLive={editor.onLive}
            onCommit={editor.onCommit}
          />
        </button>
        <button
          type="button"
          class="cg-label cg-label-btn {activeRegion === 'highlights' ? 'is-active' : ''}"
          onclick={() => (activeRegion = 'highlights')}
        >
          Highlights
        </button>
      </div>
    </div>
  {:else}
    <div class="flex flex-col items-center gap-1.5 py-2">
      <HueWheel
        bind:hue={editor.edits.color.color_grade.global.hue}
        bind:sat={editor.edits.color.color_grade.global.sat}
        size={160}
        onLive={editor.onLive}
        onCommit={editor.onCommit}
      />
      <div class="cg-label">Global</div>
    </div>
  {/if}

  <div class="flex flex-col gap-2.5 border-t border-white/10 pt-2">
    <button
      type="button"
      class="flex items-center gap-1 px-1 text-[10px] uppercase tracking-wide text-immich-dark-fg/60 hover:text-immich-dark-fg transition-colors"
      onclick={() => (adjustOpen = !adjustOpen)}
    >
      <Icon path={adjustOpen ? mdiChevronDown : mdiChevronRight} size={12} />
      Adjust — {REGION_LABELS[activeRegion]}
    </button>
    {#if adjustOpen}
      <div class="flex items-center justify-between text-[10px] font-mono tabular-nums text-immich-dark-fg/50 px-1">
        <span>H: {Math.round(activeRegionData.hue)}</span>
        <span>S: {Math.round(activeRegionData.sat)}</span>
        <button
          type="button"
          class="flex items-center gap-1 text-immich-dark-fg/40 hover:text-immich-dark-fg/80 transition-colors"
          title="Reset {REGION_LABELS[activeRegion]}"
          onclick={() => resetRegion(activeRegion)}
        >
          <Icon path={mdiRestore} size={12} />
          Reset
        </button>
      </div>
      <SliderRow
        label="Hue"
        bind:value={activeRegionData.hue}
        min={0}
        max={360}
        step={1}
        onLive={editor.onLive}
        onCommit={editor.onCommit}
        format={(v: number) => v.toFixed(0)}
        gradient={hueGradient}
      />
      <SliderRow
        label="Saturation"
        bind:value={activeRegionData.sat}
        min={0}
        max={100}
        step={1}
        onLive={editor.onLive}
        onCommit={editor.onCommit}
        format={(v: number) => v.toFixed(0)}
        gradient={satGradient}
      />
      <SliderRow
        label="Luminance"
        bind:value={activeRegionData.lum}
        min={-50}
        max={50}
        step={1}
        onLive={editor.onLive}
        onCommit={editor.onCommit}
        format={(v: number) => v.toFixed(0)}
        gradient={lumGradient}
      />
    {/if}
  </div>

  <div class="flex flex-col gap-2.5 border-t border-white/10 pt-2">
    <SliderRow
      label="Balance"
      bind:value={editor.edits.color.color_grade.balance}
      min={-100}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
      gradient={balanceGradient}
    />
    <SliderRow
      label="Blending"
      bind:value={editor.edits.color.color_grade.blend}
      min={0}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
      gradient={blendingGradient}
    />
  </div>
</div>

<style>
  .cg-triangle {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-areas:
      'top top'
      'sh hi';
    column-gap: 12px;
    row-gap: 8px;
    justify-items: center;
    padding: 4px 0;
  }
  .cg-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }
  .cg-cell-top {
    grid-area: top;
  }
  .cg-wheel-btn {
    padding: 3px;
    border-radius: 9999px;
    background: transparent;
    border: 1px solid transparent;
    transition: border-color 0.15s, box-shadow 0.15s;
    cursor: pointer;
  }
  .cg-wheel-btn:hover {
    border-color: rgba(255, 255, 255, 0.2);
  }
  .cg-wheel-btn.is-active {
    border-color: rgba(255, 255, 255, 0.55);
    box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.15);
  }
  .cg-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: rgba(255, 255, 255, 0.55);
  }
  .cg-label-btn {
    background: transparent;
    border: 0;
    padding: 2px 6px;
    border-radius: 4px;
    cursor: pointer;
    transition: color 0.15s, background-color 0.15s;
  }
  .cg-label-btn:hover {
    color: rgba(255, 255, 255, 0.85);
  }
  .cg-label-btn.is-active {
    color: rgba(255, 255, 255, 0.95);
    background-color: rgba(255, 255, 255, 0.08);
  }
</style>
