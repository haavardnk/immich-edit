<script lang="ts">
  import SearchableSelect from '$lib/components/SearchableSelect.svelte';
  import { Label } from '@immich/ui';
  import { searchSuggestions } from '$lib/api/search';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';
  import {
    EXIF_LABELS,
    exifSummary,
    suggestionQuery,
    type ExifFilterKey
  } from '$lib/browse/exifFilters';

  let { key }: { key: ExifFilterKey } = $props();

  let suggestions = $state<string[]>([]);
  let request = 0;

  const label = $derived(EXIF_LABELS[key]);
  const selected = $derived(browseControls[key]);
  const options = $derived([
    ...suggestions,
    ...selected.filter((value) => !suggestions.includes(value))
  ]);
  const multiple = $derived(immichCapabilities.structuredSearch);
  const query = $derived(suggestionQuery(key, browseControls));

  $effect(() => {
    const id = ++request;
    searchSuggestions(query)
      .then((values) => {
        if (id !== request) return;
        suggestions = values;
        const kept = browseControls[key].filter((value) => values.includes(value));
        if (kept.length < browseControls[key].length) browseControls[key] = kept;
      })
      .catch(() => undefined);
  });
</script>

<div class="flex min-w-0 flex-col gap-1">
  <Label {label} size="small" />
  <SearchableSelect
    {options}
    bind:selected={
      () => browseControls[key],
      (values) => {
        browseControls[key] = values;
      }
    }
    getId={(value) => value}
    getLabel={(value) => value}
    {multiple}
    hideSelected={false}
    closeOnSelect={!multiple}
    placeholder="Select {label.toLowerCase()}"
    {label}
    summary={exifSummary(selected)}
  />
</div>
