<script lang="ts">
  import { page } from '$app/state';
  import SearchField from '#lib/components/SearchField.svelte';
  import IdGroup from '#lib/components/browse/filters/IdGroup.svelte';
  import { routeScope, structuredRoute } from '#lib/browse/browseRoutes.ts';
  import { ID_EXCLUSIONS, ID_SCOPES, type IdFilterKey } from '#lib/browse/idFilters.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import type { FilterKey } from '#lib/stores/browseControls.svelte.ts';

  let { supports }: { supports: (key: FilterKey) => boolean } = $props();

  const ORDER: IdFilterKey[] = ['people', 'albums', 'tags'];

  let query = $state('');

  const keys = $derived(ORDER.filter((key) => supports(key)));
  const structured = $derived(structuredRoute(page.url));
  const scope = $derived<Partial<Record<string, string>>>(routeScope(page.url) ?? {});

  $effect(() => {
    for (const key of keys) library.request(key);
  });
</script>

<SearchField bind:value={query} label="Search albums, people and tags" placeholder="Search" />
{#each keys as key (key)}
  <IdGroup
    {key}
    query={query.trim()}
    excludable={supports(ID_EXCLUSIONS[key].key)}
    {structured}
    scopeId={scope[ID_SCOPES[key]]}
  />
{/each}
