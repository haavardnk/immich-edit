<script lang="ts">
  import { commandShortLabel } from '#lib/commands/command.ts';
  import { loupeCompare, loupeSurvey } from '#lib/commands/loupe/navigation.ts';
  import { PANE_ACTIONS } from '#lib/commands/loupe/panes.ts';
  import CommandIconButton from '#lib/components/CommandIconButton.svelte';
  import CommandPill from '#lib/components/CommandPill.svelte';
  import { compare } from '#lib/stores/compare.svelte.ts';
  import { Icon } from '@immich/ui';

  let { variant = 'icon' }: { variant?: 'icon' | 'pill' } = $props();

  const mode = $derived(compare.mode === 'compare' ? loupeCompare : loupeSurvey);
  const actions = $derived(
    PANE_ACTIONS.filter((command) => command.contexts.some((context) => context === compare.mode))
  );
</script>

{#if variant === 'pill'}
  {#each actions as command (command.id)}
    <CommandPill {command} />
  {/each}
{:else}
  <div
    class="flex h-6 items-center gap-1.5 rounded bg-ghost px-2.5 text-[10px] text-light-950/65 touch:h-9 touch:text-xs"
  >
    <Icon icon={mode.icon} size="13px" class="text-primary" aria-hidden="true" />
    <span class="font-medium text-light-950/85">{commandShortLabel(mode)}</span>
  </div>
  {#each actions as command (command.id)}
    <CommandIconButton {command} size="tiny" variant="ghost" class="touch:size-11" />
  {/each}
{/if}
