<script lang="ts">
  import LibraryLoad from '#lib/components/library/LibraryLoad.svelte';
  import PersonFace from '#lib/components/library/PersonFace.svelte';
  import { library } from '#lib/stores/library.svelte.ts';
</script>

<LibraryLoad view="people" label="people">
  {#if library.people.length === 0}
    <p class="px-4 text-xs text-dark/65 sm:px-5" role="status">No people</p>
  {:else}
    <ul
      aria-label="People"
      class="flex gap-3.5 overflow-x-auto px-4 pt-1 pb-1.5 scrollbar-hidden sm:px-5"
    >
      {#each library.people as person (person.id)}
        <li><PersonFace {person} /></li>
      {/each}
    </ul>
  {/if}
</LibraryLoad>
