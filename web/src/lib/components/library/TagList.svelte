<script lang="ts">
  import { library } from '$lib/stores/library.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiTagOutline } from '@mdi/js';
</script>

{#if library.tags.length === 0}
  <div class="p-3 text-xs text-immich-dark-fg/30">no tags</div>
{:else}
  <div class="flex flex-col gap-0.5 p-1">
    {#each library.tags as t (t.id)}
      <a
        href={`/tags/${t.id}`}
        class="flex items-center gap-2.5 py-1.5 px-2.5 rounded-lg hover:bg-white/5 transition-colors"
      >
        <Icon path={mdiTagOutline} size={16} class="opacity-40 flex-none" />
        <span class="truncate text-[13px] leading-tight pr-2">{t.name}</span>
      </a>
    {/each}
  </div>
{/if}
