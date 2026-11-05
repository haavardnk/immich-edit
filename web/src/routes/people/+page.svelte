<script lang="ts">
  import { onMount } from 'svelte';
  import LibraryLoad from '#lib/components/library/LibraryLoad.svelte';
  import PersonFace from '#lib/components/library/PersonFace.svelte';
  import PageFrame from '#lib/components/shell/PageFrame.svelte';
  import { library } from '#lib/stores/library.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';

  const loaded = $derived(library.isLoaded('people'));

  onMount(() => {
    library.request('people');
  });
</script>

<PageFrame
  title="People"
  count={loaded ? countOf(library.people.length, 'person', 'people') : undefined}
>
  <LibraryLoad view="people" label="people">
    {#if library.people.length === 0}
      <p class="px-6 py-16 text-center text-sm text-dark/65" role="status">No people</p>
    {:else}
      <ul
        aria-label="People"
        class="grid grid-cols-[repeat(auto-fill,minmax(5.5rem,1fr))] justify-items-center gap-2 p-3 sm:p-5"
      >
        {#each library.people as person (person.id)}
          <li><PersonFace {person} /></li>
        {/each}
      </ul>
    {/if}
  </LibraryLoad>
</PageFrame>
