<script lang="ts">
  import RatingBadge from '#lib/components/browse/RatingBadge.svelte';
  import type { ClusterItem } from '#lib/browse/tileContent.ts';
  import { OVERLAY_ICON_PATHS } from '#lib/browse/overlayIcons.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Icon } from '@immich/ui';

  let {
    items,
    counts,
    ratingClass = '',
    class: className = ''
  }: {
    items: readonly ClusterItem[];
    counts: boolean;
    ratingClass?: string;
    class?: string;
  } = $props();

  const rightFirst = $derived([...items].reverse());
</script>

{#snippet counted(icon: string, count: number, name: string)}
  <span role="img" aria-label={countOf(count, name)} class="flex h-3.5 items-center gap-0.5">
    <Icon {icon} size="12px" aria-hidden="true" />
    {#if counts && count > 1}
      <span class="text-nano" aria-hidden="true">{count}</span>
    {/if}
  </span>
{/snippet}

<span
  data-testid="tile-cluster"
  class="ml-auto flex h-3.5 max-w-full min-w-0 shrink flex-row-reverse flex-wrap content-start items-center gap-x-1.5 overflow-hidden text-micro font-medium whitespace-nowrap {className}"
>
  {#each rightFirst as item (item.key)}
    {#if item.key === 'rating'}
      <RatingBadge rating={item.rating} class="h-3.5 {ratingClass}" />
    {:else if item.key === 'copy'}
      <span class="flex h-3.5 items-center gap-0.5 text-white/75">
        <Icon icon={OVERLAY_ICON_PATHS.copy} size="12px" aria-hidden="true" />
        {#if counts}
          <span class="text-nano">{item.text}</span>
        {:else}
          <span class="sr-only">{item.text}</span>
        {/if}
      </span>
    {:else if item.key === 'format'}
      <span
        role="img"
        aria-label="{item.text} file"
        class="flex h-3.25 items-center rounded-xs px-0.75 text-nano leading-none font-bold tracking-wide ring-1 ring-white/70 ring-inset"
      >
        <span aria-hidden="true">{item.text}</span>
      </span>
    {:else if item.key === 'albums'}
      {@render counted(OVERLAY_ICON_PATHS.albums, item.count, 'album')}
    {:else if item.key === 'tags'}
      {@render counted(OVERLAY_ICON_PATHS.tags, item.count, 'tag')}
    {:else}
      <span
        role="img"
        aria-label={item.key === 'location' ? 'Location' : 'Description'}
        class="flex h-3.5 items-center"
      >
        <Icon icon={OVERLAY_ICON_PATHS[item.key]} size="12px" aria-hidden="true" />
      </span>
    {/if}
  {/each}
</span>
