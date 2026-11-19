<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import FilterToggle from '#lib/components/browse/FilterToggle.svelte';
  import SelectModeButton from '#lib/components/browse/SelectModeButton.svelte';
  import ViewOptions from '#lib/components/browse/ViewOptions.svelte';
  import { IconButton } from '@immich/ui';
  import { mdiDotsHorizontal } from '@mdi/js';

  let { sortBasis }: { sortBasis: 'capture' | 'edit' | null } = $props();

  let open = $state(false);
</script>

<FilterToggle />
<SelectModeButton class="size-11" />
<Popover
  {open}
  align="end"
  onOpenChange={(v) => (open = v)}
  appearance="panel"
  title="View options"
  contentClass="flex min-w-64 flex-col gap-3 p-3"
  sheetClass="flex flex-col gap-4 px-4 py-3"
  tooltip="View options"
>
  {#snippet trigger(popoverProps)}
    <IconButton
      size="medium"
      variant="ghost"
      color="secondary"
      shape="round"
      class="size-11"
      icon={mdiDotsHorizontal}
      title=""
      aria-label="View options"
      {...popoverProps}
    />
  {/snippet}
  <ViewOptions {sortBasis} />
</Popover>
