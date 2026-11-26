<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import BulkActionRow from '#lib/components/browse/bulk/BulkActionRow.svelte';
  import BulkLocationMenu from '#lib/components/browse/bulk/BulkLocationMenu.svelte';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import type { BulkMenuAction } from '#lib/browse/bulk/bulkMenu.ts';
  import { bulkMetadata } from '#lib/browse/bulk/bulkMetadata.svelte.ts';
  import { Icon } from '@immich/ui';
  import { mdiChevronRight, mdiMapMarkerOutline } from '@mdi/js';
  import type { Snippet } from 'svelte';

  let {
    open,
    onOpenChange,
    onDone,
    anchor = 'top',
    tooltip,
    trigger
  }: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onDone?: () => void;
    anchor?: 'top' | 'right';
    tooltip?: string;
    trigger: Snippet<[Record<string, unknown>]>;
  } = $props();

  let location = $state(false);

  function finish(): void {
    location = false;
    onOpenChange(false);
    onDone?.();
  }

  function pick(action: BulkMenuAction): void {
    finish();
    action.run();
  }

  function setOpen(next: boolean): void {
    if (!next) location = false;
    onOpenChange(next);
  }
</script>

<Popover
  {open}
  {anchor}
  align="end"
  onOpenChange={setOpen}
  contentClass="w-60 py-1"
  {tooltip}
  {trigger}
>
  <div class="flex flex-col" role="group" aria-label="Metadata">
    <BulkActionRow action={bulkMetadata.captureTime} onPick={pick} />
    <BulkActionRow action={bulkMetadata.description} onPick={pick} />
    <BulkLocationMenu
      open={location}
      anchor="right"
      onOpenChange={(next) => (location = next)}
      onDone={finish}
    >
      {#snippet trigger(props)}
        <MenuRow
          {...props}
          icon={mdiMapMarkerOutline}
          label="Location"
          aria-label="Location"
          pressed={location}
          disabled={bulkActions.metaBusy}
        >
          <Icon icon={mdiChevronRight} size="14px" class="shrink-0 opacity-70" aria-hidden="true" />
        </MenuRow>
      {/snippet}
    </BulkLocationMenu>
    {#if bulkMetadata.copy || bulkMetadata.paste}
      <div class="my-1 h-px bg-hairline" role="separator"></div>
    {/if}
    {#if bulkMetadata.copy}
      <BulkActionRow action={bulkMetadata.copy} onPick={pick} />
    {/if}
    {#if bulkMetadata.paste}
      <BulkActionRow action={bulkMetadata.paste} onPick={pick} />
    {/if}
  </div>
</Popover>
