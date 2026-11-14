<script lang="ts">
  import {
    commandShortLabel,
    commandTitle,
    isEnabled,
    isShown,
    runCommand
  } from '#lib/commands/command.ts';
  import { LOUPE_VIEW_MODE_MENU } from '#lib/commands/menus.ts';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';

  let { class: className = '' }: { class?: string } = $props();

  const modes = LOUPE_VIEW_MODE_MENU.flat();
  const options = $derived(
    modes.filter(isShown).map((command) => ({
      value: command.id,
      label: commandShortLabel(command),
      icon: command.icon,
      title: commandTitle(command),
      disabled: !isEnabled(command)
    }))
  );
  const value = $derived(modes.find((command) => command.pressed?.())?.id ?? '');

  function choose(id: string): void {
    const command = modes.find((candidate) => candidate.id === id);
    if (command) runCommand(command);
  }
</script>

<div class={className}>
  <SegmentedChoice {options} {value} label="View mode" appearance="photo" fill onChange={choose} />
</div>
