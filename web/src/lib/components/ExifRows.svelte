<script lang="ts">
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { exifDetailRows, type Dated } from '#lib/utils/exif.ts';

  let { asset }: { asset: Dated | null } = $props();

  const rows = $derived(exifDetailRows(asset));
</script>

<div class="flex flex-col gap-1 text-[11px] min-w-56">
  {#if rows.length === 0}
    <div class="text-dark/65 italic">No EXIF data</div>
  {:else}
    {#each rows as r (r.key)}
      <div class="flex justify-between gap-3">
        <span class="text-dark/65">{r.key}</span>
        <TruncationTooltip text={r.value}>
          {#snippet child({ props })}
            <span {...props} class="truncate font-mono text-dark">{r.value}</span>
          {/snippet}
        </TruncationTooltip>
      </div>
    {/each}
  {/if}
</div>
