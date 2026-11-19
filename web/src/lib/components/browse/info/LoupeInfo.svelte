<script lang="ts">
  import { reveal } from '#lib/motion/reveal.ts';
  import PhotoInfoPanel from '#lib/components/browse/info/PhotoInfoPanel.svelte';
  import PhotoInfoSheet from '#lib/components/browse/info/PhotoInfoSheet.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import type { AssetSummary } from '#lib/types/album.ts';

  let { asset }: { asset: AssetSummary } = $props();
</script>

{#if layout.compact}
  {#if browseView.loupeInfoOpen}
    <PhotoInfoSheet {asset} onclose={() => browseView.setLoupeInfoOpen(false)} />
  {/if}
{:else if browseView.loupeInfoOpen}
  <div class="flex h-full flex-none" transition:reveal={{ axis: 'x' }}>
    <PhotoInfoPanel {asset} onclose={() => browseView.setLoupeInfoOpen(false)} />
  </div>
{/if}
