<script lang="ts">
  import BulkActionBar from '#lib/components/browse/bulk/BulkActionBar.svelte';
  import TouchBulkBar from '#lib/components/browse/bulk/TouchBulkBar.svelte';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { layout } from '#lib/device/layout.svelte.ts';

  let { placement }: { placement: 'float' | 'dock' } = $props();

  const docked = $derived(layout.mode === 'touch');
</script>

{#if placement === 'dock'}
  {#if docked && bulkScope.selecting}
    <TouchBulkBar />
  {/if}
{:else if !docked && bulkScope.selecting}
  <BulkActionBar />
{/if}
