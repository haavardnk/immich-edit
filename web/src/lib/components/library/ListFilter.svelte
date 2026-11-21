<script lang="ts" generics="T">
  import { ListFilterState } from '#lib/browse/listFilter.svelte.ts';
  import type { ListNoun } from '#lib/browse/listFilter.ts';
  import ListFilterField from '#lib/components/library/ListFilterField.svelte';
  import ListFilterResults from '#lib/components/library/ListFilterResults.svelte';
  import type { Snippet } from 'svelte';

  let {
    items,
    nameOf,
    noun,
    class: className = '',
    children
  }: {
    items: readonly T[];
    nameOf: (item: T) => string;
    noun: ListNoun;
    class?: string;
    children: Snippet<[readonly T[], string]>;
  } = $props();

  const filter = new ListFilterState(
    () => items,
    (item: T) => nameOf(item),
    () => noun
  );
</script>

<ListFilterField {filter} class="py-1 {className}" />
<ListFilterResults {filter} class={className} {children} />
