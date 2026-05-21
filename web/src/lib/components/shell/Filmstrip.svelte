<script lang="ts">
  import { page } from '$app/state';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { thumbUrl } from '$lib/api/assets';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiChevronDown, mdiChevronUp } from '@mdi/js';

  const currentId = $derived(page.params.id ?? null);

  const assets = $derived(browsing.assets);
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
    {#if ui.filmstripCollapsed}
      <button
        class="w-full flex items-center justify-center h-5 hover:bg-white/5 transition-colors"
        onclick={ui.toggleFilmstrip}
        aria-label="expand filmstrip"
        title="Filmstrip"
      >
        <Icon path={mdiChevronUp} size={14} class="opacity-40" />
      </button>
    {:else}
      <div class="relative">
        <button
          class="absolute right-2 top-1 z-10 p-0.5 rounded hover:bg-white/10 transition-colors"
          onclick={ui.toggleFilmstrip}
          aria-label="collapse filmstrip"
          title="Collapse"
        >
          <Icon path={mdiChevronDown} size={14} class="opacity-40" />
        </button>
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
  </div>
{/if}
