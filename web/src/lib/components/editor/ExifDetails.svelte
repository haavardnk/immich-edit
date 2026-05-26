<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { exifDetailRows } from '$lib/util/exif';

  const rows = $derived(exifDetailRows(editor.asset?.exifInfo ?? null));
</script>

<div class="flex flex-col gap-1 text-[11px] min-w-56">
  {#if rows.length === 0}
    <div class="text-immich-dark-fg/30 italic">No EXIF data</div>
  {:else}
    {#each rows as r (r.key)}
      <div class="flex justify-between gap-3">
        <span class="text-immich-dark-fg/50">{r.key}</span>
        <span class="text-immich-dark-fg/90 font-mono truncate" title={r.value}>{r.value}</span>
      </div>
    {/each}
  {/if}
</div>
