<script lang="ts">
  import { page } from '$app/state';
  import { album } from '$lib/stores/album.svelte';
  import { thumbUrl } from '$lib/api/assets';
  import type { AssetSummary } from '$lib/types/album';

  const currentId = $derived(page.params.id ?? null);

  const assets = $derived<AssetSummary[]>(album.current?.assets ?? []);
  const currentIndex = $derived(assets.findIndex((a) => a.id === currentId));

  let scrollContainer: HTMLDivElement | undefined = $state();

  $effect(() => {
    if (currentIndex >= 0 && scrollContainer) {
      const el = scrollContainer.children[currentIndex] as HTMLElement | undefined;
      el?.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' });
    }
  });
</script>

{#if assets.length > 0}
  <div class="border-t border-white/5 bg-immich-dark-gray flex-none">
    <div
      class="flex gap-1 px-2 py-2 overflow-x-auto scrollbar-hidden"
      bind:this={scrollContainer}
    >
      {#each assets as asset, i (asset.id)}
        {@const isCurrent = asset.id === currentId}
        <a
          href={`/assets/${asset.id}`}
          class="flex-none w-16 h-16 rounded-lg overflow-hidden transition-all {isCurrent ? 'ring-2 ring-immich-dark-primary opacity-100' : 'opacity-50 hover:opacity-80'}"
          title={asset.originalFileName}
        >
          <img
            src={thumbUrl(asset.id)}
            alt=""
            loading="lazy"
            class="w-full h-full object-cover"
          />
        </a>
      {/each}
    </div>
  </div>
{/if}
