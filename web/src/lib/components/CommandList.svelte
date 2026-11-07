<script lang="ts">
  import MenuItem from '#lib/components/MenuItem.svelte';
  import { commandLabel, isEnabled, isShown, pressHandlers } from '#lib/commands/command.ts';
  import type { Menu } from '#lib/commands/menus.ts';
  import type { Command, IconCommand } from '#lib/commands/types.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';

  type Props = {
    menu: Menu;
    onrun?: () => void;
    rowClass?: (command: Command) => string | undefined;
  };

  let { menu, onrun, rowClass }: Props = $props();

  const groups = $derived(
    menu.map((group) => group.filter(isShown)).filter((group) => group.length > 0)
  );
</script>

{#snippet row(command: IconCommand)}
  <MenuItem
    icon={command.icon}
    label={commandLabel(command)}
    keys={command.bind ? keysFor(command.bind) : undefined}
    pressed={command.pressed?.()}
    disabled={!isEnabled(command)}
    {...pressHandlers(command, onrun)}
  />
{/snippet}

{#each groups as group, index (index)}
  {#if index > 0}
    <div class="my-1 border-t border-dark/10"></div>
  {/if}
  {#each group as command (command.id)}
    {@const cls = rowClass?.(command)}
    {#if cls}
      <div class="contents {cls}">{@render row(command)}</div>
    {:else}
      {@render row(command)}
    {/if}
  {/each}
{/each}
