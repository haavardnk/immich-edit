<script lang="ts">
  import ParamSlider from '#lib/components/editor/controls/ParamSlider.svelte';
  import type { ParamDef } from '#lib/editor/params/types.ts';
  import { quickLabel } from '#lib/editor/quickSettings.ts';

  let { params }: { params: readonly ParamDef[] } = $props();
</script>

<div data-quick class="flex flex-col">
  {#each params as param (param.id)}
    <ParamSlider {param} label={quickLabel(param)} />
  {/each}
</div>
