<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import HueWheel from '$lib/components/editor/controls/HueWheel.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiRestore } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { HSL_BAND_NAMES, HSL_BAND_COLORS } from '$lib/types/edits';

  let activeBand = $state(0);

  const CG_REGIONS = [
    { key: 'shadows' as const, label: 'Shadows' },
    { key: 'midtones' as const, label: 'Midtones' },
    { key: 'highlights' as const, label: 'Highlights' },
    { key: 'global' as const, label: 'Global' }
  ];

  type RegionKey = (typeof CG_REGIONS)[number]['key'];

  function resetRegion(key: RegionKey): void {
    const reg = editor.edits.color.color_grade[key];
    reg.hue = 0;
    reg.sat = 0;
    reg.lum = 0;
    editor.onCommit();
  }

  function resetAllGrading(): void {
    const cg = editor.edits.color.color_grade;
    for (const r of CG_REGIONS) {
      cg[r.key].hue = 0;
      cg[r.key].sat = 0;
      cg[r.key].lum = 0;
    }
    cg.balance = 0;
    cg.blend = 0;
    editor.onCommit();
  }
</script>

<div class="flex flex-col gap-2.5">
  <SliderRow
    label="Saturation"
    bind:value={editor.edits.basic.saturation}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />
  <SliderRow
    label="Vibrance"
    bind:value={editor.edits.basic.vibrance}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />

  <div class="flex flex-col gap-2 pt-2 mt-1 border-t border-white/10">
    <div class="text-[10px] uppercase tracking-wide text-immich-dark-fg/60 px-1">HSL</div>
    <div class="grid grid-cols-8 gap-1">
      {#each HSL_BAND_NAMES as name, i (name)}
        <button
          type="button"
          class="h-6 rounded ring-1 ring-white/10 hover:ring-white/40 transition-shadow {activeBand === i ? 'ring-2 ring-white/80' : ''}"
          style="background-color: {HSL_BAND_COLORS[i]}"
          title={name}
          onclick={() => (activeBand = i)}
        ></button>
      {/each}
    </div>
    <div class="text-[11px] text-immich-dark-fg/70 px-1">{HSL_BAND_NAMES[activeBand]}</div>
    <SliderRow
      label="Hue"
      bind:value={editor.edits.color.hsl.bands[activeBand].hue}
      min={-100}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Sat"
      bind:value={editor.edits.color.hsl.bands[activeBand].sat}
      min={-100}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Lum"
      bind:value={editor.edits.color.hsl.bands[activeBand].lum}
      min={-100}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
  </div>

  <div class="flex flex-col gap-3 pt-2 mt-1 border-t border-white/10">
    <div class="flex items-center justify-between px-1">
      <div class="text-[10px] uppercase tracking-wide text-immich-dark-fg/60">Color grading</div>
      <button
        type="button"
        class="flex items-center gap-1 text-[10px] text-immich-dark-fg/60 hover:text-immich-dark-fg transition-colors"
        title="Reset color grading"
        onclick={resetAllGrading}
      >
        <Icon path={mdiRestore} size={12} />
        Reset
      </button>
    </div>
    {#each CG_REGIONS as r (r.key)}
      <div class="flex gap-3 items-center">
        <HueWheel
          bind:hue={editor.edits.color.color_grade[r.key].hue}
          bind:sat={editor.edits.color.color_grade[r.key].sat}
          onLive={editor.onLive}
          onCommit={editor.onCommit}
        />
        <div class="flex flex-col gap-1.5 flex-1 min-w-0">
          <div class="flex items-center justify-between">
            <div class="text-[11px] text-immich-dark-fg/70">{r.label}</div>
            <button
              type="button"
              class="text-immich-dark-fg/40 hover:text-immich-dark-fg/80 transition-colors"
              title="Reset {r.label}"
              onclick={() => resetRegion(r.key)}
            >
              <Icon path={mdiRestore} size={12} />
            </button>
          </div>
          <SliderRow
            label="Lum"
            bind:value={editor.edits.color.color_grade[r.key].lum}
            min={-50}
            max={50}
            step={1}
            onLive={editor.onLive}
            onCommit={editor.onCommit}
            format={(v: number) => v.toFixed(0)}
          />
        </div>
      </div>
    {/each}
    <SliderRow
      label="Balance"
      bind:value={editor.edits.color.color_grade.balance}
      min={-100}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Blend"
      bind:value={editor.edits.color.color_grade.blend}
      min={0}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
  </div>
</div>
