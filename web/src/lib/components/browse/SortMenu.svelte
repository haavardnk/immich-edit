<script lang="ts">
  import Popover from '$lib/components/Popover.svelte';
  import { SEGMENTED_GROUP_CLASS, SEGMENTED_ITEM_CLASS } from '$lib/components/browse/segmented';
  import { browseControls, type SortDir, type SortField } from '$lib/stores/browseControls.svelte';
  import { IconButton, Label } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';
  import {
    mdiSortCalendarAscending,
    mdiSortCalendarDescending,
    mdiSortClockAscending,
    mdiSortClockDescending
  } from '@mdi/js';

  let open = $state(false);

  const ICONS: Record<SortField, Record<SortDir, string>> = {
    taken: { asc: mdiSortCalendarAscending, desc: mdiSortCalendarDescending },
    added: { asc: mdiSortClockAscending, desc: mdiSortClockDescending }
  };

  const label = $derived(
    `Sort: date ${browseControls.sortField}, ${browseControls.sortDir === 'asc' ? 'oldest' : 'newest'} first`
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
      <RadioGroup.Item value="taken" class={SEGMENTED_ITEM_CLASS}>Date taken</RadioGroup.Item>
      <RadioGroup.Item value="added" class={SEGMENTED_ITEM_CLASS}>Date added</RadioGroup.Item>
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
      <RadioGroup.Item value="desc" class={SEGMENTED_ITEM_CLASS}>Newest first</RadioGroup.Item>
      <RadioGroup.Item value="asc" class={SEGMENTED_ITEM_CLASS}>Oldest first</RadioGroup.Item>
    </RadioGroup.Root>
  </div>
</Popover>
