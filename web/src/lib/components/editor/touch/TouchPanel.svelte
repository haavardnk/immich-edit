<script lang="ts">
  import PresetStrip from '#lib/components/preset/PresetStrip.svelte';
  import ExportPanel from '#lib/panels/Export.svelte';
  import DevelopViews from './DevelopViews.svelte';
  import GroupChips from './GroupChips.svelte';
  import SettingsSearch from './SettingsSearch.svelte';
  import TouchCrop from './crop/TouchCrop.svelte';
  import TouchMasks from './masks/TouchMasks.svelte';
  import { touchUi } from '#lib/stores/touchUi.svelte.ts';
  import { valueEntry } from '#lib/stores/valueEntry.svelte.ts';
  import { touchMode } from './modes';
  import TouchRetouch from './retouch/TouchRetouch.svelte';

  let { stacked = false, onretap }: { stacked?: boolean; onretap?: () => void } = $props();

  const mode = $derived(touchMode());
</script>

{#if mode === 'presets'}
  <div class="min-h-0 flex-1 overflow-y-auto">
    <PresetStrip />
  </div>
{:else if mode === 'develop'}
  {#if touchUi.searching}
    <SettingsSearch />
  {:else}
    {#if stacked || !valueEntry.target}
      <GroupChips grid={stacked} {onretap} />
    {/if}
    <DevelopViews {stacked} />
  {/if}
{:else if mode === 'geometry'}
  <TouchCrop {stacked} />
{:else if mode === 'masks'}
  <TouchMasks {stacked} />
{:else if mode === 'retouch'}
  <TouchRetouch {stacked} />
{:else}
  <div class="min-h-0 flex-1 overflow-y-auto overscroll-contain px-3 py-2 scrollbar-hidden">
    <ExportPanel />
  </div>
{/if}
