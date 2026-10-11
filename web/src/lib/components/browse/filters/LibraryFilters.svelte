<script lang="ts">
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import {
    browseControls,
    type RejectedFilter,
    type Visibility
  } from '$lib/stores/browseControls.svelte';
  import { Field, Select } from '@immich/ui';

  let { favoriteLocked = false }: { favoriteLocked?: boolean } = $props();

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

<Field label="Visibility" size="tiny">
  <Select
    size="tiny"
    class="editor-compact-select editor-compact-field"
    options={visibilityOptions}
    value={browseControls.visibility}
    onChange={(v) => (browseControls.visibility = v)}
  />
</Field>

<Field label="Rejected" size="tiny">
  <Select
    size="tiny"
    class="editor-compact-select editor-compact-field"
    options={rejectedOptions}
    value={browseControls.rejected}
    onChange={(v) => (browseControls.rejected = v)}
  />
</Field>

{#if !favoriteLocked}
  <CheckboxRow
    label="Favorites only"
    checked={browseControls.favoriteOnly}
    onChange={(checked) => (browseControls.favoriteOnly = checked)}
  />
{/if}
