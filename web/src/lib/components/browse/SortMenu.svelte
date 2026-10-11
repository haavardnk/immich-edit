<script lang="ts">
  import Popover from '$lib/components/Popover.svelte';
  import { SEGMENTED_GROUP_CLASS, SEGMENTED_ITEM_CLASS } from '$lib/components/browse/segmented';
  import { browseControls, type SortDir, type SortField } from '$lib/stores/browseControls.svelte';
  import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';
  import { IconButton, Label } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';
  import {
    mdiSortAscending,
    mdiSortCalendarAscending,
    mdiSortCalendarDescending,
    mdiSortClockAscending,
    mdiSortClockDescending,
    mdiSortDescending,
    mdiSortNumericAscending,
    mdiSortNumericDescending
  } from '@mdi/js';

  let open = $state(false);

  const ICONS: Record<SortField, Record<SortDir, string>> = {
    taken: { asc: mdiSortCalendarAscending, desc: mdiSortCalendarDescending },
    added: { asc: mdiSortClockAscending, desc: mdiSortClockDescending },
    rating: { asc: mdiSortNumericAscending, desc: mdiSortNumericDescending },
    size: { asc: mdiSortAscending, desc: mdiSortDescending }
  };

  const FIELDS: Record<SortField, string> = {
    taken: 'Date taken',
    added: 'Date added',
    rating: 'Rating',
    size: 'Size'
  };

  const DIRECTIONS: Record<SortField, Record<SortDir, string>> = {
    taken: { desc: 'Newest first', asc: 'Oldest first' },
    added: { desc: 'Newest first', asc: 'Oldest first' },
    rating: { desc: 'Highest first', asc: 'Lowest first' },
    size: { desc: 'Largest first', asc: 'Smallest first' }
  };

  const fields = $derived(
    (Object.keys(FIELDS) as SortField[]).filter(
      (field) => (field !== 'rating' && field !== 'size') || immichCapabilities.structuredSearch
    )
  );

  const directions = $derived(DIRECTIONS[browseControls.sortField]);

  const label = $derived(
    `Sort: ${FIELDS[browseControls.sortField].toLowerCase()}, ${directions[browseControls.sortDir].toLowerCase()}`
  );
</script>

<Popover
  {open}
  align="end"
  onOpenChange={(v) => (open = v)}
  contentClass="flex min-w-55 flex-col gap-2.5 p-3"
  tooltip={label}
>
  {#snippet trigger(popoverProps)}
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      icon={ICONS[browseControls.sortField][browseControls.sortDir]}
      title=""
      aria-label={label}
      {...popoverProps}
    />
  {/snippet}
  <div class="flex flex-col gap-1">
    <Label label="Sort by" size="small" />
    <RadioGroup.Root
      bind:value={
        () => browseControls.sortField, (v) => browseControls.setSortField(v as SortField)
      }
      orientation="horizontal"
      aria-label="Sort by"
      class="{SEGMENTED_GROUP_CLASS} h-10"
    >
      {#each fields as field (field)}
        <RadioGroup.Item value={field} class={SEGMENTED_ITEM_CLASS}>
          {FIELDS[field]}
        </RadioGroup.Item>
      {/each}
    </RadioGroup.Root>
  </div>
  <div class="flex flex-col gap-1">
    <Label label="Order" size="small" />
    <RadioGroup.Root
      bind:value={() => browseControls.sortDir, (v) => browseControls.setSortDir(v as SortDir)}
      orientation="horizontal"
      aria-label="Order"
      class="{SEGMENTED_GROUP_CLASS} h-10"
    >
      <RadioGroup.Item value="desc" class={SEGMENTED_ITEM_CLASS}>{directions.desc}</RadioGroup.Item>
      <RadioGroup.Item value="asc" class={SEGMENTED_ITEM_CLASS}>{directions.asc}</RadioGroup.Item>
    </RadioGroup.Root>
  </div>
</Popover>
