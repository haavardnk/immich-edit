<script lang="ts">
  import { isEnabled, runCommand } from '#lib/commands/command.ts';
  import { copyEdits, pasteEdits, pasteOnly, resetDevelop } from '#lib/commands/editor/edits.ts';
  import type { Command } from '#lib/commands/types.ts';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import { clipboard } from '#lib/stores/clipboard.svelte.ts';

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
  {#if clipboard.has && clipboard.source}
    <span class="max-w-[45%] min-w-0 truncate text-[13px] text-dark/65"
      >from {clipboard.source}</span
    >
  {/if}
</MenuRow>
<MenuRow
  icon={pasteOnly.icon}
  label="Paste only…"
  class="hidden touch:flex"
  aria-label="Paste only…"
  disabled={!isEnabled(pasteOnly)}
  onclick={() => run(pasteOnly)}
/>
<MenuRow
  icon={resetDevelop.icon}
  label="Reset Develop"
  class="hidden touch:flex"
  aria-label="Reset Develop"
  danger
  disabled={!isEnabled(resetDevelop)}
  onclick={() => run(resetDevelop)}
/>
