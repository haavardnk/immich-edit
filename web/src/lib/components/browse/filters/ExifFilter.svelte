<script lang="ts">
  import SearchableSelect from '$lib/components/SearchableSelect.svelte';
  import { searchSuggestions } from '$lib/api/search';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';
  import {
    EXIF_LABELS,
    UNKNOWN_EXIF,
    exclusiveUnknown,
    exifLabel,
    exifSummary,
    suggestionQuery,
    type ExifFilterKey
  } from '$lib/browse/exifFilters';

  let { key }: { key: ExifFilterKey } = $props();

  let suggestions = $state<string[]>([]);
  let request = 0;

  const label = $derived(EXIF_LABELS[key]);
  const selected = $derived(browseControls.filters[key]);
  const options = $derived([
    ...suggestions,
    ...selected.filter((value) => !suggestions.includes(value))
  ]);
  const multiple = $derived(immichCapabilities.structuredSearch);
  const query = $derived(suggestionQuery(key, browseControls.filters));

  $effect(() => {
    const id = ++request;
    searchSuggestions(query)
      .then((values) => {
        if (id !== request) return;
        suggestions = values.map((value) => value ?? UNKNOWN_EXIF);
        browseControls.prune(key, suggestions);
      })
      .catch(() => undefined);
  });
</script>

<div class="panel-row min-h-7 items-start">
  <span class="editor-compact-label flex h-7 items-center select-none">{label}</span>
  <div class="col-span-2 col-start-2 min-w-0">
    <SearchableSelect
      compact
      color="neutral"
      {options}
      bind:selected={
        () => browseControls.filters[key],
        (values) => {
          browseControls.filters[key] = exclusiveUnknown(values);
        }
      }
      getId={(value) => value}
      getLabel={exifLabel}
      {multiple}
      hideSelected={false}
      closeOnSelect={!multiple}
      placeholder="Select {label.toLowerCase()}"
      {label}
      summary={exifSummary(selected)}
    />
  </div>
</div>
