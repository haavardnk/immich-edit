<script lang="ts">
  import { page } from '$app/state';
  import { library } from '$lib/stores/library.svelte';
  import { personThumbUrl } from '$lib/api/people';

  const activeId = $derived(
    page.url.pathname.startsWith('/people/') ? (page.params.id ?? null) : null
  );
</script>

{#if library.people.length === 0}
  <div class="px-5 py-3 text-xs text-muted" role="status">No people</div>
{:else}
  <div class="flex flex-col gap-0.5 p-1">
    {#each library.people as p (p.id)}
      <a
        href={`/people/${p.id}`}
        aria-current={p.id === activeId ? 'page' : undefined}
        class="flex items-center gap-2.5 rounded-e-full py-1.5 ps-2.5 pe-4 outline-none transition-colors focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary {p.id ===
        activeId
          ? 'bg-primary/15 text-primary'
          : 'hover:bg-subtle hover:text-primary'}"
      >
        <div class="h-8 w-8 flex-none overflow-hidden rounded-full bg-gray-800">
          <img
            src={personThumbUrl(p.id)}
            alt=""
            loading="lazy"
            class="w-full h-full object-cover"
          />
        </div>
        <span class="min-w-0 flex-1 truncate pe-2 text-[13px] leading-tight"
          >{p.name || 'Unknown'}</span
        >
        {#if p.assetCount != null}
          <span class="flex-none text-[11px] text-muted tabular-nums">{p.assetCount}</span>
        {/if}
      </a>
    {/each}
  </div>
{/if}
