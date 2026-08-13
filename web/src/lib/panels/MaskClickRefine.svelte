<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { refineWith, stopClickTool } from './maskTools';

  let { layerId }: { layerId: string } = $props();

  const active = $derived(editor.clickTool.active && editor.clickTool.layerId === layerId);
</script>

<div class="mt-1 flex items-center justify-between gap-2 px-1">
  <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Click to refine</div>
  <div class="flex items-center gap-2">
    <div class="flex rounded ring-1 ring-white/10 overflow-hidden text-[10px]">
      <button
        type="button"
        class="px-2 leading-5 transition-colors {active && !editor.clickTool.negative
          ? 'bg-white/15 text-immich-dark-fg'
          : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
        title="Click the photo to add that area to this shape"
        onclick={() => refineWith(layerId, false)}>Add</button
      >
      <button
        type="button"
        class="px-2 leading-5 transition-colors {active && editor.clickTool.negative
          ? 'bg-white/15 text-immich-dark-fg'
          : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
        title="Click the photo to cut that area out of this shape"
        onclick={() => refineWith(layerId, true)}>Remove</button
      >
    </div>
    {#if active}
      <button
        type="button"
        class="text-[10px] text-immich-dark-fg/50 hover:text-immich-dark-fg"
        onclick={stopClickTool}>Done</button
      >
    {/if}
  </div>
</div>
{#if editor.maskGenerating}
  <div class="px-1 text-[10px] text-immich-dark-fg/40">Working…</div>
{/if}
