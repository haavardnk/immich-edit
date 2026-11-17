<script lang="ts">
  import FilterList from '#lib/components/browse/filters/FilterList.svelte';
  import FilterListRow from '#lib/components/browse/filters/FilterListRow.svelte';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { personThumbUrl } from '#lib/api/people.ts';
  import {
    ID_EXCLUSIONS,
    ID_LABELS,
    ID_MATCHES,
    ID_MATCH_LABELS,
    ID_NOUNS,
    idFilter,
    type IdFilterKey
  } from '#lib/browse/idFilters.ts';
  import { idOptions, type IdOption } from '#lib/browse/idOptions.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { library } from '#lib/stores/library.svelte.ts';

  let {
    key,
    excludable,
    structured,
    scopeId,
    full = false,
    onAll
  }: {
    key: IdFilterKey;
    excludable: boolean;
    structured: boolean;
    scopeId: string | undefined;
    full?: boolean;
    onAll?: () => void;
  } = $props();

  const MATCH_OPTIONS = ID_MATCHES.map((value) => ({ value, label: ID_MATCH_LABELS[value] }));

  const all = $derived(idOptions(key));
  const options = $derived((all ?? []).filter((option) => option.id !== scopeId));
  const current = $derived(browseControls.filters[key]);
  const exclusion = $derived(ID_EXCLUSIONS[key]);

  $effect(() => {
    library.request(key);
  });

  $effect(() => {
    if (all)
      browseControls.pruneIds(
        key,
        all.map((option) => option.id)
      );
  });

  function toggle(id: string, checked: boolean): void {
    const ids = checked ? [...current.ids, id] : current.ids.filter((entry) => entry !== id);
    browseControls.filters[key] = idFilter(ids, current.match);
  }
</script>

{#snippet avatar(option: IdOption)}
  <img
    src={personThumbUrl(option.id)}
    alt=""
    loading="lazy"
    class="size-5 shrink-0 rounded-full bg-light-200 object-cover touch:size-8"
  />
{/snippet}

{#snippet match()}
  {#if structured && current.ids.length > 0}
    <SegmentedChoice
      appearance="mini"
      options={MATCH_OPTIONS}
      value={current.match}
      label="{ID_LABELS[key]} match"
      onChange={(next) => (browseControls.filters[key] = idFilter(current.ids, next))}
    />
  {/if}
{/snippet}

{#snippet exclude()}
  {#if excludable}
    <FilterListRow
      label={exclusion.label}
      muted
      checked={browseControls.filters[exclusion.key]}
      onChange={(checked) => (browseControls.filters[exclusion.key] = checked)}
    />
  {/if}
{/snippet}

{#if all}
  <FilterList
    items={options}
    label={ID_LABELS[key]}
    noun={ID_NOUNS[key]}
    isSelected={(id) => current.ids.includes(id)}
    onToggle={toggle}
    {full}
    {onAll}
    leading={key === 'people' ? avatar : undefined}
    header={match}
    footer={exclude}
  />
{/if}
