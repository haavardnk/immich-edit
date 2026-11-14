<script lang="ts">
  import AvatarChip from '#lib/components/chips/AvatarChip.svelte';
  import Chip from '#lib/components/chips/Chip.svelte';
  import ChipList from '#lib/components/chips/ChipList.svelte';
  import HighlightedText from '#lib/components/HighlightedText.svelte';
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
  import { filterByName, listFilterStatus } from '#lib/browse/listFilter.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { mdiAccountOffOutline } from '@mdi/js';

  let {
    key,
    query,
    excludable,
    structured,
    scopeId
  }: {
    key: IdFilterKey;
    query: string;
    excludable: boolean;
    structured: boolean;
    scopeId: string | undefined;
  } = $props();

  const MATCH_OPTIONS = ID_MATCHES.map((value) => ({ value, label: ID_MATCH_LABELS[value] }));

  const label = $derived(ID_LABELS[key]);
  const exclusion = $derived(ID_EXCLUSIONS[key]);
  const all = $derived(idOptions(key));
  const options = $derived((all ?? []).filter((option) => option.id !== scopeId));
  const matches = $derived(
    query ? filterByName(options, query, (option) => option.label) : options
  );
  const current = $derived(browseControls.filters[key]);
  const excluded = $derived(browseControls.filters[exclusion.key]);

  $effect(() => {
    if (all)
      browseControls.pruneIds(
        key,
        all.map((option) => option.id)
      );
  });

  function toggle(id: string, pressed: boolean): void {
    const ids = pressed ? [...current.ids, id] : current.ids.filter((entry) => entry !== id);
    browseControls.filters[key] = idFilter(ids, current.match);
  }

  function exclude(pressed: boolean): void {
    browseControls.filters[exclusion.key] = pressed;
  }

  function picked(option: IdOption): boolean {
    return current.ids.includes(option.id);
  }
</script>

<div class="flex flex-col gap-1.5 touch:gap-2">
  <div class="flex min-h-6.5 items-center gap-2 touch:min-h-10">
    <span class="text-xs text-dark/65 select-none touch:text-[13px]" aria-hidden="true">
      {label}
    </span>
    {#if structured && current.ids.length > 0}
      <SegmentedChoice
        appearance="mini"
        options={MATCH_OPTIONS}
        value={current.match}
        label="{label} match"
        onChange={(match) => (browseControls.filters[key] = idFilter(current.ids, match))}
      />
    {/if}
    <span class="flex-1"></span>
    {#if query && all}
      <span role="status" class="text-[11.5px] text-dark/65 touch:text-[12.5px]">
        {listFilterStatus(matches.length, options.length, ID_NOUNS[key])}
      </span>
    {/if}
  </div>
  {#key query}
    <ChipList
      items={matches}
      key={(option) => option.id}
      selected={picked}
      limit={key === 'people' ? 7 : 8}
      {label}
    >
      {#snippet chip(option)}
        {#if key === 'people'}
          <AvatarChip
            name={option.label}
            image={personThumbUrl(option.id)}
            {query}
            pressed={picked(option)}
            onPressedChange={(pressed) => toggle(option.id, pressed)}
          />
        {:else}
          <Chip pressed={picked(option)} onPressedChange={(pressed) => toggle(option.id, pressed)}>
            <HighlightedText text={option.label} {query} />
          </Chip>
        {/if}
      {/snippet}
      {#if excludable}
        {#if key === 'people'}
          <AvatarChip
            name={exclusion.label}
            icon={mdiAccountOffOutline}
            pressed={excluded}
            onPressedChange={exclude}
          />
        {:else}
          <Chip tone="exclusion" pressed={excluded} onPressedChange={exclude}>
            {exclusion.label}
          </Chip>
        {/if}
      {/if}
    </ChipList>
  {/key}
</div>
