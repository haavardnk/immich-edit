<script lang="ts">
  import ParamSlider from '#lib/components/editor/controls/ParamSlider.svelte';
  import SectionHeader from '#lib/components/editor/controls/SectionHeader.svelte';
  import {
    grainAmount,
    grainRoughness,
    grainSize,
    noGrain,
    noVignette,
    vignetteAmount,
    vignetteFeather,
    vignetteMidpoint,
    vignetteRoundness
  } from '#lib/editor/params/effects.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { NEUTRAL_EFFECTS } from '#lib/types/edits.ts';

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
    <SectionHeader
      title="Vignette"
      section="vignette"
      modified={!noVignette(editor.edits)}
      onReset={resetVignette}
    />
    <ParamSlider param={vignetteAmount} />
    <ParamSlider param={vignetteMidpoint} />
    <ParamSlider param={vignetteFeather} />
    <ParamSlider param={vignetteRoundness} />
  </div>
  <div class="flex flex-col gap-1 pt-1.5">
    <SectionHeader
      title="Grain"
      section="grain"
      modified={!noGrain(editor.edits)}
      onReset={resetGrain}
    />
    <ParamSlider param={grainAmount} />
    <ParamSlider param={grainSize} />
    <ParamSlider param={grainRoughness} />
  </div>
</div>
