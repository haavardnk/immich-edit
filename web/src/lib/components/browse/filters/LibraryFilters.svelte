<script lang="ts">
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import {
    browseControls,
    type FilterKey,
    type RejectedFilter,
    type Visibility
  } from '$lib/stores/browseControls.svelte';
  import { Field, Select } from '@immich/ui';

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
</script>

{#if supports('visibility')}
  <Field label="Visibility" size="tiny">
    <Select
      size="tiny"
      class="editor-compact-select editor-compact-field"
      options={visibilityOptions}
      value={browseControls.filters.visibility}
      onChange={(v) => (browseControls.filters.visibility = v)}
    />
  </Field>
{/if}

<Field label="Rejected" size="tiny">
  <Select
    size="tiny"
    class="editor-compact-select editor-compact-field"
    options={rejectedOptions}
    value={browseControls.filters.rejected}
    onChange={(v) => (browseControls.filters.rejected = v)}
  />
</Field>

{#if !favoriteLocked}
  <div class="pt-1">
    <CheckboxRow
      label="Favorites only"
      checked={browseControls.filters.favoriteOnly}
      onChange={(checked) => (browseControls.filters.favoriteOnly = checked)}
    />
  </div>
{/if}
