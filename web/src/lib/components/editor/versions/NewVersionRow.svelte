<script lang="ts">
  import { commandLabel, isEnabled, pressHandlers } from '#lib/commands/command.ts';
  import { virtualCopy } from '#lib/commands/editor/edits.ts';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { mdiPlus } from '@mdi/js';

  let { onrun }: { onrun: () => void } = $props();

  const label = $derived(commandLabel(virtualCopy));
</script>

<MenuRow
  icon={mdiPlus}
  {label}
  keys={virtualCopy.bind ? keysFor(virtualCopy.bind) : undefined}
  class="touch:h-auto touch:py-4 touch:font-medium touch:text-primary"
  aria-label={label}
  disabled={!isEnabled(virtualCopy)}
  {...pressHandlers(virtualCopy, onrun)}
>
  <span class="hidden shrink-0 text-[13px] font-normal text-dark/65 touch:inline">from Current</span
  >
</MenuRow>
