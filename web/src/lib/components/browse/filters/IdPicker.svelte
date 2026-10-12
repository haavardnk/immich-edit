<script lang="ts">
  import { page } from '$app/state';
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import SearchableSelect from '$lib/components/SearchableSelect.svelte';
  import {
    segmentedControlClass,
    segmentedRadioItemClass
  } from '$lib/components/editor/controls/segmentedControl';
  import { routeScope, structuredRoute } from '$lib/browse/browseRoutes';
  import {
    ID_EXCLUSIONS,
    ID_LABELS,
    ID_MATCHES,
    ID_MATCH_LABELS,
    ID_SCOPES,
    idFilter,
    idSummary,
    type IdFilterKey,
    type IdMatch
  } from '$lib/browse/idFilters';
  import { idName, idOptions } from '$lib/browse/idOptions';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { RadioGroup } from 'bits-ui';

  let { key, excludable }: { key: IdFilterKey; excludable: boolean } = $props();

  const label = $derived(ID_LABELS[key]);
  const exclusion = $derived(ID_EXCLUSIONS[key]);
  const current = $derived(browseControls.filters[key]);
  const all = $derived(idOptions(key));
  const scopeId = $derived.by(() => {
    const scope: Partial<Record<string, string>> = routeScope(page.url) ?? {};
    return scope[ID_SCOPES[key]];
  });
  const options = $derived((all ?? []).filter((option) => option.id !== scopeId));
  const structured = $derived(structuredRoute(page.url));

  $effect(() => {
    if (all)
      browseControls.pruneIds(
        key,
        all.map((option) => option.id)
      );
  });
</script>

<div class="panel-row min-h-7 items-start">
  <span class="editor-compact-label flex h-7 items-center select-none">{label}</span>
  <div class="col-span-2 col-start-2 flex min-w-0 flex-col gap-1">
    <SearchableSelect
      compact
      color="neutral"
      {options}
      bind:selected={
        () => browseControls.filters[key].ids,
        (ids) => {
          browseControls.filters[key] = idFilter(ids, browseControls.filters[key].match);
        }
      }
      getId={(option) => option.id}
      getLabel={(option) => option.label}
      hideSelected={false}
      closeOnSelect={false}
      placeholder="Select {label.toLowerCase()}"
      {label}
      summary={idSummary(current.ids, (id) => idName(key, id))}
    />
    {#if structured && current.ids.length > 0}
      <RadioGroup.Root
        bind:value={
          () => current.match,
          (match) => {
            browseControls.filters[key] = idFilter(current.ids, match as IdMatch);
          }
        }
        orientation="horizontal"
        aria-label="{label} match"
        class={segmentedControlClass}
      >
        {#each ID_MATCHES as match (match)}
          <RadioGroup.Item value={match} class="{segmentedRadioItemClass} flex-1">
            {ID_MATCH_LABELS[match]}
          </RadioGroup.Item>
        {/each}
      </RadioGroup.Root>
    {/if}
    {#if excludable}
      <CheckboxRow
        label={exclusion.label}
        checked={browseControls.filters[exclusion.key]}
        onChange={(checked) => (browseControls.filters[exclusion.key] = checked)}
      />
    {/if}
  </div>
</div>
