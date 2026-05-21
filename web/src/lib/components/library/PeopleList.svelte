<script lang="ts">
  import { library } from '$lib/stores/library.svelte';
  import { personThumbUrl } from '$lib/api/people';
</script>

{#if library.people.length === 0}
  <div class="p-3 text-xs text-immich-dark-fg/30">no people</div>
{:else}
  <div class="flex flex-col gap-0.5 p-1">
    {#each library.people as p (p.id)}
      <a
        href={`/people/${p.id}`}
        class="flex items-center gap-2.5 py-1.5 px-2.5 rounded-lg hover:bg-white/5 transition-colors"
      >
        <div class="w-8 h-8 rounded-full bg-white/5 overflow-hidden flex-none">
          <img
            src={personThumbUrl(p.id)}
            alt=""
            loading="lazy"
            class="w-full h-full object-cover"
          />
        </div>
        <span class="truncate text-[13px] leading-tight pr-2">{p.name || 'Unknown'}</span>
      </a>
    {/each}
  </div>
{/if}
