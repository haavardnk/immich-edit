<script lang="ts">
  import FilterList from '#lib/components/browse/filters/FilterList.svelte';
  import FilterListRow from '#lib/components/browse/filters/FilterListRow.svelte';
  import { searchSuggestions } from '#lib/api/search.ts';
  import { EXIF_PLURALS, drillCrumbs, drillLevel, pickExif } from '#lib/browse/exifDrill.ts';
  import {
    EXIF_LABELS,
    UNKNOWN_EXIF,
    suggestionQuery,
    type ExifPath
  } from '#lib/browse/exifFilters.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiChevronRight } from '@mdi/js';

  let {
    keys,
    label,
    full = false,
    onAll
  }: { keys: Readonly<ExifPath>; label: string; full?: boolean; onAll?: () => void } = $props();

  let pinned = $state<number | null>(null);
  let suggestions = $state<string[]>([]);
  let request = 0;
  let ancestorRequest = 0;

  const filters = $derived(browseControls.filters);
  const auto = $derived(drillLevel(keys, filters));
  const level = $derived(pinned === null ? auto : Math.min(pinned, auto));
  const key = $derived(keys[level] ?? keys[0]);
  const selected = $derived(filters[key]);
  const crumbs = $derived(drillCrumbs(keys, filters, level));
  const multiple = $derived(immichCapabilities.structuredSearch);
  const query = $derived(suggestionQuery(key, filters));
  const values = $derived([
    ...suggestions.filter((value) => value !== UNKNOWN_EXIF),
    ...selected.filter((value) => value !== UNKNOWN_EXIF && !suggestions.includes(value))
  ]);
  const unknown = $derived(suggestions.includes(UNKNOWN_EXIF) || selected.includes(UNKNOWN_EXIF));
  const items = $derived(values.map((value) => ({ id: value, label: value })));
  const noun = $derived({
    one: EXIF_LABELS[key].toLowerCase(),
    many: EXIF_PLURALS[key].toLowerCase()
  });

  $effect(() => {
    const id = ++request;
    const field = key;
    suggestions = [];
    searchSuggestions(query)
      .then((found) => {
        if (id !== request) return;
        suggestions = found.map((value) => value ?? UNKNOWN_EXIF);
        browseControls.prune(field, suggestions);
      })
      .catch(() => undefined);
  });

  const ancestors = $derived(
    keys
      .slice(0, level)
      .flatMap((entry) =>
        filters[entry].length > 0 ? [{ key: entry, query: suggestionQuery(entry, filters) }] : []
      )
  );

  $effect(() => {
    const id = ++ancestorRequest;
    for (const { key: entry, query: parentQuery } of ancestors) {
      searchSuggestions(parentQuery)
        .then((found) => {
          if (id !== ancestorRequest) return;
          browseControls.prune(
            entry,
            found.map((value) => value ?? UNKNOWN_EXIF)
          );
        })
        .catch(() => undefined);
    }
  });

  function pick(value: string): void {
    const next = pickExif(keys, filters, level, value, multiple);
    for (const entry of keys) {
      const picked = next[entry];
      if (picked && JSON.stringify(picked) !== JSON.stringify(filters[entry])) {
        browseControls.filters[entry] = picked;
      }
    }
    pinned = null;
  }
</script>

{#if crumbs.length > 0}
  <nav
    aria-label="{label} path"
    class="-ml-1.5 flex h-6.5 min-w-0 items-center text-xs whitespace-nowrap text-dark/65 touch:-ml-2 touch:h-10 touch:text-sm"
  >
    {#each crumbs as crumb, index (crumb.level)}
      {#if index > 0}
        <Icon icon={mdiChevronRight} size="14px" class="shrink-0 opacity-55" aria-hidden="true" />
      {/if}
      {#if crumb.level === level}
        <span aria-current="location" class="truncate px-1.5 font-medium text-light-950 touch:px-2">
          {crumb.label}
        </span>
      {:else}
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          class="h-6 min-w-0 shrink px-1.5 text-xs font-normal touch:h-9 touch:px-2 touch:text-sm {index >
          0
            ? 'text-primary'
            : 'text-dark/65'}"
          onclick={() => (pinned = crumb.level)}
        >
          <span class="truncate">{crumb.label}</span>
        </Button>
      {/if}
    {/each}
  </nav>
{/if}
{#snippet unknownRow()}
  {#if unknown}
    <FilterListRow
      label="Unknown {EXIF_LABELS[key].toLowerCase()}"
      muted
      checked={selected.includes(UNKNOWN_EXIF)}
      onChange={() => pick(UNKNOWN_EXIF)}
    />
  {/if}
{/snippet}

{#key key}
  <FilterList
    {items}
    label={EXIF_PLURALS[key]}
    {noun}
    isSelected={(value) => selected.includes(value)}
    onToggle={(value) => pick(value)}
    {full}
    {onAll}
    footer={unknownRow}
  />
{/key}
