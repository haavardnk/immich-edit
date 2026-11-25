<script lang="ts">
  import type { Job } from '#lib/api/jobs.ts';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import BulkActionRow from '#lib/components/browse/bulk/BulkActionRow.svelte';
  import BulkApplyPreset from '#lib/components/browse/bulk/BulkApplyPreset.svelte';
  import { bulkEdits } from '#lib/browse/bulk/bulkEdits.svelte.ts';
  import type { BulkMenuAction } from '#lib/browse/bulk/bulkMenu.ts';
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { Icon } from '@immich/ui';
  import { mdiChevronRight } from '@mdi/js';
  import type { Snippet } from 'svelte';

  let {
    open,
    onOpenChange,
    onDone,
    anchor = 'top',
    tooltip,
    trigger: opener
  }: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onDone?: () => void;
    anchor?: 'top' | 'right';
    tooltip?: string;
    trigger: Snippet<[Record<string, unknown>]>;
  } = $props();

  let flyout = $state(false);
  let menu = $state<HTMLElement | null>(null);

  const apply = $derived(bulkEdits.applyPreset);

  function setOpen(next: boolean): void {
    if (!next) flyout = false;
    onOpenChange(next);
  }

  function finish(): void {
    setOpen(false);
    onDone?.();
  }

  function pick(action: BulkMenuAction): void {
    finish();
    action.run();
  }

  function applied(job: Job): void {
    finish();
    jobs.showJob(job.id);
  }
</script>

<Popover
  {open}
  {anchor}
  align="end"
  onOpenChange={setOpen}
  contentClass="w-72 py-1"
  {tooltip}
  trigger={opener}
>
  <div bind:this={menu} class="flex flex-col" role="group" aria-label="Edits">
    <Popover
      open={flyout}
      anchor="right"
      align="end"
      appearance="panel"
      contentClass="w-80"
      customAnchor={menu}
      onOpenChange={(next) => (flyout = next)}
    >
      {#snippet trigger(props)}
        <MenuRow
          {...props}
          icon={apply.icon}
          label={apply.label}
          aria-label={apply.label}
          pressed={flyout}
          disabled={apply.disabled}
        >
          <Icon icon={mdiChevronRight} size="14px" class="shrink-0 opacity-70" aria-hidden="true" />
        </MenuRow>
      {/snippet}
      <BulkApplyPreset onApplied={applied} />
    </Popover>
    <BulkActionRow action={bulkEdits.paste} onPick={pick} />
    <BulkActionRow action={bulkEdits.pasteOnly} onPick={pick} />
    <BulkActionRow action={bulkEdits.copy} onPick={pick} />
    <div class="my-1 h-px bg-hairline" role="separator"></div>
    <BulkActionRow action={bulkEdits.reset} onPick={pick} />
  </div>
</Popover>
