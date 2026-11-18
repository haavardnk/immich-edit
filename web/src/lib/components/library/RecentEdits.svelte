<script lang="ts">
  import { assetThumbUrl } from '#lib/api/assets.ts';
  import { recentEdits } from '#lib/browse/recentEdits.ts';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
  import { editorHref } from '#lib/editor/navigation.ts';
  import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';

  let { returnPath }: { returnPath: string } = $props();

  const entries = $derived(recentEdits(editedThumbs.entries));
</script>

{#if entries.length === 0}
  <p class="px-4 text-xs text-dark/65 sm:px-5" role="status">No edited photos yet</p>
{:else}
  <ul
    aria-label="Recently edited"
    class="flex gap-1 overflow-x-auto px-4 py-1 scrollbar-hidden sm:px-5"
  >
    {#each entries as entry, index (entry.id)}
      {@const name = entry.asset?.originalFileName}
      <li class="shrink-0">
        <TooltipLink
          always
          tooltip={name ?? 'Open recent edit'}
          href={editorHref(entry.id, returnPath)}
          aria-label={name ? `Open ${name}` : `Open recent edit ${index + 1}`}
          class="no-callout block h-28 w-42 overflow-hidden rounded-lg bg-light-200 outline-none focus-visible:ring-2 focus-visible:ring-primary touch:h-32 touch:w-48"
        >
          <img src={assetThumbUrl(entry.id)} alt="" loading="lazy" class="size-full object-cover" />
        </TooltipLink>
      </li>
    {/each}
  </ul>
{/if}
