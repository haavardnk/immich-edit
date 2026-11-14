<script lang="ts">
  import ParamSlider from '#lib/components/editor/controls/ParamSlider.svelte';
  import { gradeBalance, gradeBlend } from '#lib/editor/params/colorGrading.ts';
  import GradeRegionPicker from './colorGrading/GradeRegionPicker.svelte';
  import GradeWheels from './colorGrading/GradeWheels.svelte';

  let { single = false }: { single?: boolean } = $props();
</script>

<div class="flex flex-col gap-1">
  {#if single}
    <GradeRegionPicker />
  {:else}
    <GradeWheels />
  {/if}

  <div class="flex flex-col gap-1 border-t border-hairline pt-1">
    {#if single}
      <ParamSlider param={gradeBlend} />
      <ParamSlider param={gradeBalance} />
    {:else}
      <ParamSlider param={gradeBalance} />
      <ParamSlider param={gradeBlend} />
    {/if}
  </div>
</div>
