<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import BulkLocationRows from '#lib/components/browse/bulk/BulkLocationRows.svelte';
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

  function finish(): void {
    onOpenChange(false);
    onDone?.();
  }
</script>

<Popover {open} {anchor} align="end" {onOpenChange} contentClass="w-72 py-1" {tooltip} {trigger}>
  <BulkLocationRows onDone={finish} />
</Popover>
