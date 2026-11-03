<script lang="ts">
  import ParamSlider from '#lib/components/editor/controls/ParamSlider.svelte';
  import SectionHeader from '#lib/components/editor/controls/SectionHeader.svelte';
  import { commitSectionReset } from '#lib/components/editor/controls/paramReset.ts';
  import {
    GRAIN,
    grainAmount,
    grainRoughness,
    grainSize,
    noGrain,
    noVignette,
    VIGNETTE,
    vignetteAmount,
    vignetteFeather,
    vignetteMidpoint,
    vignetteRoundness
  } from '#lib/editor/params/effects.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
</script>

<div class="flex flex-col divide-y divide-dark/10">
  <div class="flex flex-col gap-1 pb-1.5">
    <SectionHeader
      title="Vignette"
      section="vignette"
      modified={!noVignette(editor.edits)}
      onReset={() => commitSectionReset(VIGNETTE)}
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
      onReset={() => commitSectionReset(GRAIN)}
    />
    <ParamSlider param={grainAmount} />
    <ParamSlider param={grainSize} />
    <ParamSlider param={grainRoughness} />
  </div>
</div>
