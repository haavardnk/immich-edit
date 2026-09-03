<script lang="ts">
  import EditSlider from '$lib/components/editor/controls/EditSlider.svelte';
  import SectionHeader from '$lib/components/editor/controls/SectionHeader.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { NEUTRAL_EFFECTS } from '$lib/types/edits';

  const vignetteInactive = $derived(editor.edits.effects.vignette_amount === 0);
  const grainInactive = $derived(editor.edits.effects.grain_amount === 0);

  function resetVignette(): void {
    editor.edits.effects.vignette_amount = NEUTRAL_EFFECTS.vignette_amount;
    editor.edits.effects.vignette_midpoint = NEUTRAL_EFFECTS.vignette_midpoint;
    editor.edits.effects.vignette_feather = NEUTRAL_EFFECTS.vignette_feather;
    editor.edits.effects.vignette_roundness = NEUTRAL_EFFECTS.vignette_roundness;
    void editor.onCommit('Reset Vignette');
  }

  function resetGrain(): void {
    editor.edits.effects.grain_amount = NEUTRAL_EFFECTS.grain_amount;
    editor.edits.effects.grain_size = NEUTRAL_EFFECTS.grain_size;
    editor.edits.effects.grain_roughness = NEUTRAL_EFFECTS.grain_roughness;
    void editor.onCommit('Reset Grain');
  }
</script>

<div class="flex flex-col divide-y divide-dark/10">
  <div class="flex flex-col gap-1 pb-1.5">
    <SectionHeader title="Vignette" modified={!vignetteInactive} onReset={resetVignette} />
    <EditSlider
      label="Amount"
      commitAction="Vignette Amount"
      bind:value={editor.edits.effects.vignette_amount}
      min={-100}
      max={100}
    />
    <EditSlider
      label="Midpoint"
      commitAction="Vignette Midpoint"
      bind:value={editor.edits.effects.vignette_midpoint}
      min={0}
      max={100}
      defaultValue={50}
      disabled={vignetteInactive}
    />
    <EditSlider
      label="Feather"
      commitAction="Vignette Feather"
      bind:value={editor.edits.effects.vignette_feather}
      min={0}
      max={100}
      defaultValue={50}
      disabled={vignetteInactive}
    />
    <EditSlider
      label="Roundness"
      commitAction="Vignette Roundness"
      bind:value={editor.edits.effects.vignette_roundness}
      min={-100}
      max={100}
      disabled={vignetteInactive}
    />
  </div>
  <div class="flex flex-col gap-1 pt-1.5">
    <SectionHeader title="Grain" modified={!grainInactive} onReset={resetGrain} />
    <EditSlider
      label="Amount"
      commitAction="Grain Amount"
      bind:value={editor.edits.effects.grain_amount}
      min={0}
      max={100}
    />
    <EditSlider
      label="Size"
      commitAction="Grain Size"
      bind:value={editor.edits.effects.grain_size}
      min={0}
      max={100}
      defaultValue={25}
      disabled={grainInactive}
    />
    <EditSlider
      label="Roughness"
      commitAction="Grain Roughness"
      bind:value={editor.edits.effects.grain_roughness}
      min={0}
      max={100}
      defaultValue={50}
      disabled={grainInactive}
    />
  </div>
</div>
