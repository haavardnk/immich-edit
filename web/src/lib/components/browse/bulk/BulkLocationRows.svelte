<script lang="ts">
  import BulkActionRow from '#lib/components/browse/bulk/BulkActionRow.svelte';
  import { bulkLocation } from '#lib/browse/bulk/bulkLocation.svelte.ts';
  import type { BulkMenuAction } from '#lib/browse/bulk/bulkMenu.ts';
  import { places } from '#lib/stores/places.svelte.ts';
  import { onMount } from 'svelte';

  let { onDone }: { onDone: () => void } = $props();

  function pick(action: BulkMenuAction): void {
    onDone();
    action.run();
  }

  onMount(() => {
    void places.ensure();
  });
</script>

{#snippet section(title: string, actions: BulkMenuAction[])}
  {#if actions.length > 0}
    <div class="flex flex-col" role="group" aria-label={title}>
      <div class="section-label px-3 pt-2 pb-1 text-dark/65 touch:px-4" aria-hidden="true">
        {title}
      </div>
      {#each actions as action, index (`${index}:${action.label}`)}
        <BulkActionRow {action} onPick={pick} />
      {/each}
    </div>
  {/if}
{/snippet}

<div class="flex flex-col" role="group" aria-label="Location">
  <BulkActionRow action={bulkLocation.choose} onPick={pick} />
  <BulkActionRow action={bulkLocation.paste} onPick={pick} />
  <BulkActionRow action={bulkLocation.copy} onPick={pick} />
  {@render section('Saved places', bulkLocation.saved)}
  {@render section('Recent', bulkLocation.recent)}
  <div class="my-1 h-px bg-hairline touch:hidden" role="separator"></div>
  <BulkActionRow action={bulkLocation.track} onPick={pick} />
</div>
