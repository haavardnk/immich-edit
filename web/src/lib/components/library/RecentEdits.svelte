<script lang="ts">
  import { assetThumbUrl } from '#lib/api/assets.ts';
  import { recentEdits } from '#lib/browse/recentEdits.ts';
  import { editorHref } from '#lib/editor/navigation.ts';
  import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';

  let { returnPath }: { returnPath: string } = $props();

  const entries = $derived(recentEdits(editedThumbs.entries));
</script>

{#if entries.length === 0}
  <p class="px-4 text-xs text-dark/65 sm:px-5" role="status">No edited photos yet</p>
{:else}
  <ul aria-label="Recently edited" class="grid grid-cols-4 gap-1.5 px-4 sm:max-w-xl sm:px-5">
    {#each entries as entry, index (entry.id)}
      <li>
        <a
          href={editorHref(entry.id, returnPath)}
          aria-label={entry.asset?.originalFileName
            ? `Open ${entry.asset.originalFileName}`
            : `Open recent edit ${index + 1}`}
          class="no-callout block aspect-square overflow-hidden rounded-md bg-light-200 outline-none focus-visible:ring-2 focus-visible:ring-primary"
        >
          <img src={assetThumbUrl(entry.id)} alt="" loading="lazy" class="size-full object-cover" />
        </a>
      </li>
    {/each}
  </ul>
{/if}
