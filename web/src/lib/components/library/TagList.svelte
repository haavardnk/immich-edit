<script lang="ts">
  import { library } from '$lib/stores/library.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { isManagedTag, toTagRef } from '$lib/reject';
  import { mdiTagOutline } from '@mdi/js';

  const tags = $derived(library.tags.filter((t) => !isManagedTag(toTagRef(t))));
</script>

{#if tags.length === 0}
  <div class="p-3 text-xs text-immich-dark-fg/30">no tags</div>
{:else}
  <div class="flex flex-col gap-0.5 p-1">
    {#each tags as t (t.id)}
      <a
        href={`/tags/${t.id}`}
        class="flex items-center gap-2.5 py-1.5 px-2.5 rounded-lg hover:bg-white/5 transition-colors"
      >
        <Icon path={mdiTagOutline} size={16} class="opacity-40 flex-none" />
        <span class="flex-1 min-w-0 truncate text-[13px] leading-tight pr-2">{t.name}</span>
        {#if t.assetCount != null}
          <span class="text-[11px] text-immich-dark-fg/30 tabular-nums flex-none"
            >{t.assetCount}</span
          >
        {/if}
      </a>
    {/each}
  </div>
{/if}
