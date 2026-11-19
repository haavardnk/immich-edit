<script lang="ts">
  import PresetStrip from '#lib/components/preset/PresetStrip.svelte';
  import ExportPanel from '#lib/panels/Export.svelte';
  import GeometryActions from '#lib/panels/GeometryActions.svelte';
  import MasksPanel from '#lib/panels/Masks.svelte';
  import RetouchPanel from '#lib/panels/Retouch.svelte';
  import TransformPanel from '#lib/panels/Transform.svelte';
  import DevelopViews from './DevelopViews.svelte';
  import GroupChips from './GroupChips.svelte';
  import { touchMode } from './modes';

  let { stacked = false, onretap }: { stacked?: boolean; onretap?: () => void } = $props();

  const mode = $derived(touchMode());
</script>

{#if mode === 'presets'}
  <div class="min-h-0 flex-1 overflow-y-auto">
    <PresetStrip />
  </div>
{:else if mode === 'develop'}
  <GroupChips grid={stacked} {onretap} />
  <DevelopViews {stacked} />
{:else}
  <div class="min-h-0 flex-1 overflow-y-auto overscroll-contain px-3 py-2 scrollbar-hidden">
    {#if mode === 'geometry'}
      <div class="flex items-center gap-1.5 pb-2"><GeometryActions /></div>
      <TransformPanel />
    {:else if mode === 'masks'}
      <MasksPanel />
    {:else if mode === 'retouch'}
      <RetouchPanel />
    {:else}
      <ExportPanel />
    {/if}
  </div>
{/if}
