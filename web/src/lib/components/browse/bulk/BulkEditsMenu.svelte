<script lang="ts">
  import type { Job } from '#lib/api/jobs.ts';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import BulkApplyPreset from '#lib/components/browse/bulk/BulkApplyPreset.svelte';
  import { bulkEdits, type BulkEditAction } from '#lib/browse/bulk/bulkEdits.svelte.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';
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

  function pick(action: BulkEditAction): void {
    finish();
    action.run();
  }

  function applied(job: Job): void {
    finish();
    jobs.showJob(job.id);
  }

  function keys(action: BulkEditAction): string | undefined {
    if (!action.bind || (action.disabled && action.detail)) return undefined;
    return keysFor(action.bind);
  }
</script>

{#snippet row(action: BulkEditAction)}
  <MenuRow
    icon={action.icon}
    label={action.label}
    aria-label={action.label}
    keys={keys(action)}
    disabled={action.disabled}
    onclick={() => pick(action)}
  >
    {#if action.detail}
      <span class="max-w-32 shrink-0 truncate text-[11px] text-dark/65">{action.detail}</span>
    {/if}
  </MenuRow>
{/snippet}

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
    {@render row(bulkEdits.paste)}
    {@render row(bulkEdits.pasteOnly)}
    {@render row(bulkEdits.copy)}
    <div class="my-1 h-px bg-dark/10" role="separator"></div>
    {@render row(bulkEdits.reset)}
  </div>
</Popover>
