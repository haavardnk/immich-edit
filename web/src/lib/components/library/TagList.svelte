<script lang="ts">
  import { page } from '$app/state';
  import { library } from '$lib/stores/library.svelte';
  import { isManagedTag, toTagRef } from '$lib/reject';
  import { Icon } from '@immich/ui';
  import { mdiTagOutline } from '@mdi/js';

  const tags = $derived(library.tags.filter((t) => !isManagedTag(toTagRef(t))));
  const activeId = $derived(
    page.url.pathname.startsWith('/tags/') ? (page.params.id ?? null) : null
  );
</script>

{#if tags.length === 0}
  <div class="px-5 py-3 text-xs text-muted" role="status">No tags</div>
{:else}
  <div class="flex flex-col gap-0.5 p-1">
    {#each tags as t (t.id)}
      <a
        href={`/tags/${t.id}`}
        aria-current={t.id === activeId ? 'page' : undefined}
        class="flex items-center gap-2.5 rounded-e-full py-2 ps-3 pe-4 outline-none transition-colors focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary {t.id ===
        activeId
          ? 'bg-primary/15 text-primary'
          : 'hover:bg-subtle hover:text-primary'}"
      >
        <Icon icon={mdiTagOutline} size="16px" class="opacity-40 flex-none" aria-hidden="true" />
        <span class="min-w-0 flex-1 truncate pe-2 text-[13px] leading-tight">{t.name}</span>
        {#if t.assetCount != null}
          <span class="flex-none text-[11px] text-muted tabular-nums">{t.assetCount}</span>
        {/if}
      </a>
    {/each}
  </div>
{/if}
