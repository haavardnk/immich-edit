<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import type { BulkMenuAction } from '#lib/browse/bulk/bulkMenu.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';

  let { action, onPick }: { action: BulkMenuAction; onPick: (action: BulkMenuAction) => void } =
    $props();

  const keys = $derived(
    !action.bind || (action.disabled && action.detail) ? undefined : keysFor(action.bind)
  );
</script>

<MenuRow
  icon={action.icon}
  label={action.label}
  aria-label={action.label}
  {keys}
  disabled={action.disabled}
  onclick={() => onPick(action)}
>
  {#if action.detail}
    <span class="max-w-32 shrink-0 truncate text-caption text-dark/65 touch:text-label">
      {action.detail}
    </span>
  {/if}
</MenuRow>
