<script lang="ts">
  import { isEnabled, runCommand } from '#lib/commands/command.ts';
  import { copyEdits, pasteEdits, resetDevelop } from '#lib/commands/editor/edits.ts';
  import type { Command } from '#lib/commands/types.ts';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import { clipboard } from '#lib/stores/clipboard.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';

  let { then }: { then: (action: () => void) => void } = $props();

  function run(command: Command): void {
    then(() => runCommand(command));
  }
</script>

<MenuRow
  icon={copyEdits.icon}
  label="Copy edits…"
  class="hidden touch:flex"
  aria-label="Copy edits…"
  disabled={!isEnabled(copyEdits)}
  onclick={() => run(copyEdits)}
/>
<MenuRow
  icon={pasteEdits.icon}
  label="Paste edits"
  class="hidden touch:flex"
  aria-label="Paste edits"
  disabled={!isEnabled(pasteEdits)}
  onclick={() => run(pasteEdits)}
>
  {#if clipboard.has}
    <span class="shrink-0 text-[13px] text-dark/65"
      >{countOf(clipboard.sectionCount, 'section')}</span
    >
  {/if}
</MenuRow>
<MenuRow
  icon={resetDevelop.icon}
  label="Reset Develop"
  class="hidden touch:flex"
  aria-label="Reset Develop"
  danger
  disabled={!isEnabled(resetDevelop)}
  onclick={() => run(resetDevelop)}
/>
