<script lang="ts">
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import type { FilterKey } from '$lib/browse/filterChips';
  import {
    browseControls,
    type RejectedFilter,
    type Visibility
  } from '$lib/stores/browseControls.svelte';
  import { Field, Select } from '@immich/ui';

  let {
    favoriteLocked = false,
    supports
  }: { favoriteLocked?: boolean; supports: (key: FilterKey) => boolean } = $props();

  const exclusions: { key: 'notInAlbum' | 'untagged' | 'noPeople'; label: string }[] = [
    { key: 'notInAlbum', label: 'Not in any album' },
    { key: 'untagged', label: 'Untagged' },
    { key: 'noPeople', label: 'No people' }
  ];

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
</script>

{#if supports('visibility')}
  <Field label="Visibility" size="tiny">
    <Select
      size="tiny"
      class="editor-compact-select editor-compact-field"
      options={visibilityOptions}
      value={browseControls.visibility}
      onChange={(v) => (browseControls.visibility = v)}
    />
  </Field>
{/if}

<Field label="Rejected" size="tiny">
  <Select
    size="tiny"
    class="editor-compact-select editor-compact-field"
    options={rejectedOptions}
    value={browseControls.rejected}
    onChange={(v) => (browseControls.rejected = v)}
  />
</Field>

<div class="grid grid-cols-2 gap-x-2 pt-1">
  {#if !favoriteLocked}
    <CheckboxRow
      label="Favorites only"
      checked={browseControls.favoriteOnly}
      onChange={(checked) => (browseControls.favoriteOnly = checked)}
    />
  {/if}

  {#each exclusions.filter(({ key }) => supports(key)) as { key, label } (key)}
    <CheckboxRow
      {label}
      checked={browseControls[key]}
      onChange={(checked) => (browseControls[key] = checked)}
    />
  {/each}
</div>
