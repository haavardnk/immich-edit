<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import SortControls from '#lib/components/browse/SortControls.svelte';
  import { SORT_ICONS, sortLabel } from '#lib/browse/sortOptions.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { IconButton } from '@immich/ui';

  let open = $state(false);

  const label = $derived(sortLabel(browseControls.sortField, browseControls.sortDir));
</script>

<Popover
  {open}
  align="end"
  onOpenChange={(v) => (open = v)}
  appearance="panel"
  contentClass="flex min-w-55 flex-col gap-2.5 p-3"
  sheetClass="flex flex-col gap-3 px-4 py-3"
  title="Sort"
  tooltip={label}
>
  {#snippet trigger(popoverProps)}
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      icon={SORT_ICONS[browseControls.sortField][browseControls.sortDir]}
      title=""
      aria-label={label}
      {...popoverProps}
    />
  {/snippet}
  <SortControls />
</Popover>
