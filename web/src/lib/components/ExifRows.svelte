<script lang="ts">
  import type { ExifInfo } from '$lib/types/asset';
  import { exifDetailRows } from '$lib/utils/exif';

  let { exif }: { exif: ExifInfo | null } = $props();

  const rows = $derived(exifDetailRows(exif));
</script>

<div class="flex flex-col gap-1 text-[11px] min-w-56">
  {#if rows.length === 0}
    <div class="text-dark/65 italic">No EXIF data</div>
  {:else}
    {#each rows as r (r.key)}
      <div class="flex justify-between gap-3">
        <span class="text-dark/65">{r.key}</span>
        <span class="truncate font-mono text-dark" title={r.value}>{r.value}</span>
      </div>
    {/each}
  {/if}
</div>
