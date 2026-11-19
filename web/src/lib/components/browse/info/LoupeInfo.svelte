<script lang="ts">
  import LoupeInfoPanel from '#lib/components/browse/info/LoupeInfoPanel.svelte';
  import LoupeInfoSheet from '#lib/components/browse/info/LoupeInfoSheet.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import type { AssetSummary } from '#lib/types/album.ts';

  let { asset }: { asset: AssetSummary } = $props();
</script>

{#if browseView.loupeInfoOpen}
  {#if layout.compact}
    <LoupeInfoSheet {asset} />
  {:else}
    <LoupeInfoPanel {asset} />
  {/if}
{/if}
