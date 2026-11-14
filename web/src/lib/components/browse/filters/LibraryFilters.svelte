<script lang="ts">
  import Chip from '#lib/components/chips/Chip.svelte';
  import ChipChoice from '#lib/components/chips/ChipChoice.svelte';
  import FilterRow from '#lib/components/browse/filters/FilterRow.svelte';
  import {
    browseControls,
    type FilterKey,
    type RejectedFilter,
    type Visibility
  } from '#lib/stores/browseControls.svelte.ts';
  import { mdiHeart, mdiHeartOutline } from '@mdi/js';

  let {
    favoriteLocked = false,
    supports
  }: { favoriteLocked?: boolean; supports: (key: FilterKey) => boolean } = $props();

  const visibilityOptions: { value: Visibility; label: string }[] = [
    { value: 'timeline', label: 'Timeline' },
    { value: 'archive', label: 'Archived' },
    { value: 'hidden', label: 'Hidden' }
  ];

  const rejectedOptions: { value: RejectedFilter; label: string }[] = [
    { value: 'any', label: 'Show' },
    { value: 'hide', label: 'Hide' },
    { value: 'only', label: 'Only' }
  ];

  const favoriteOnly = $derived(browseControls.filters.favoriteOnly);
</script>

{#if supports('visibility')}
  <FilterRow label="Show">
    <ChipChoice
      options={visibilityOptions}
      value={browseControls.filters.visibility}
      label="Show"
      onChange={(value) => (browseControls.filters.visibility = value)}
    />
  </FilterRow>
{/if}
<FilterRow label="Rejected">
  <ChipChoice
    options={rejectedOptions}
    value={browseControls.filters.rejected}
    label="Rejected"
    onChange={(value) => (browseControls.filters.rejected = value)}
  />
</FilterRow>
{#if !favoriteLocked}
  <FilterRow>
    <Chip
      pressed={favoriteOnly}
      onPressedChange={(pressed) => (browseControls.filters.favoriteOnly = pressed)}
      icon={favoriteOnly ? mdiHeart : mdiHeartOutline}
    >
      Favorites only
    </Chip>
  </FilterRow>
{/if}
