<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { ALBUM_SORTS, type AlbumSort } from '#lib/browse/albumSort.ts';
  import { IconButton } from '@immich/ui';
  import { mdiSort } from '@mdi/js';

  let { value, onChange }: { value: AlbumSort; onChange: (sort: AlbumSort) => void } = $props();

  let open = $state(false);

  const label = $derived(
    `Sort albums: ${ALBUM_SORTS.find((option) => option.value === value)?.label ?? ''}`
  );
</script>

<Popover
  {open}
  align="end"
  onOpenChange={(v) => (open = v)}
  appearance="panel"
  title="Sort albums"
  contentClass="flex flex-col p-3"
  sheetClass="flex flex-col px-4 py-3"
  tooltip={label}
>
  {#snippet trigger(popoverProps)}
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      class="touch:size-11"
      icon={mdiSort}
      title=""
      aria-label={label}
      {...popoverProps}
    />
  {/snippet}
  <SegmentedChoice options={ALBUM_SORTS} {value} label="Sort albums" {onChange} />
</Popover>
