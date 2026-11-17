<script lang="ts" generics="T extends FilterListItem">
  import SearchField from '#lib/components/SearchField.svelte';
  import FilterListRow from '#lib/components/browse/filters/FilterListRow.svelte';
  import { filterRows, type FilterListItem } from '#lib/browse/filterList.ts';
  import type { ListNoun } from '#lib/browse/listFilter.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiChevronRight } from '@mdi/js';
  import type { Snippet } from 'svelte';

  let {
    items,
    label,
    noun,
    isSelected,
    onToggle,
    full = false,
    onAll,
    leading,
    header,
    footer
  }: {
    items: readonly T[];
    label: string;
    noun: ListNoun;
    isSelected: (id: string) => boolean;
    onToggle: (id: string, checked: boolean) => void;
    full?: boolean;
    onAll?: () => void;
    leading?: Snippet<[T]>;
    header?: Snippet;
    footer?: Snippet;
  } = $props();

  let query = $state('');
  let expanded = $state(false);

  const term = $derived(query.trim());
  const rows = $derived(filterRows(items, isSelected, term, full || expanded));
  const scrolls = $derived(!full && (expanded || term !== ''));
  const allLabel = $derived(`All ${countOf(items.length, noun.one, noun.many)}`);

  function showAll(): void {
    if (onAll) onAll();
    else expanded = true;
  }
</script>

{#snippet row(item: T)}
  {#snippet lead()}
    {@render leading?.(item)}
  {/snippet}
  <FilterListRow
    label={item.label}
    count={item.count}
    query={term}
    checked={isSelected(item.id)}
    onChange={(checked) => onToggle(item.id, checked)}
    leading={leading ? lead : undefined}
  />
{/snippet}

<div class="flex flex-col gap-1 touch:gap-0">
  <div class="flex items-center gap-2 touch:mb-1.5">
    <div class="min-w-0 flex-1">
      <SearchField
        bind:value={query}
        label="Search {noun.many}"
        placeholder="Search {countOf(items.length, noun.one, noun.many)}"
      />
    </div>
    {@render header?.()}
  </div>
  <div role="group" aria-label={label} class="flex flex-col touch:gap-0">
    {#each rows.pinned as item (item.id)}
      {@render row(item)}
    {/each}
    {#if rows.pinned.length > 0 && (rows.rest.length > 0 || footer)}
      <div role="separator" class="mx-1 my-0.5 h-px shrink-0 bg-hairline touch:hidden"></div>
    {/if}
    <div
      class="flex flex-col {scrolls
        ? 'max-h-64 overflow-y-auto overscroll-contain touch:max-h-none touch:overflow-visible'
        : ''}"
    >
      {#each rows.rest as item (item.id)}
        {@render row(item)}
      {/each}
      {#if term && rows.rest.length === 0 && rows.pinned.length === 0}
        <p class="px-1 py-1.5 text-xs text-dark/65 touch:py-3 touch:text-[14px]">
          No matching {noun.many}
        </p>
      {/if}
      {@render footer?.()}
    </div>
  </div>
  {#if rows.hidden && !term}
    <Button
      size="tiny"
      variant="ghost"
      color="primary"
      class="h-7 justify-start gap-1 px-1 text-[12.5px] font-normal touch:h-12 touch:justify-between touch:px-0.5 touch:text-[15px]"
      onclick={showAll}
    >
      {allLabel}
      <Icon icon={mdiChevronRight} size="16px" aria-hidden="true" />
    </Button>
  {/if}
</div>
