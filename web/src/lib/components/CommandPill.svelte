<script lang="ts">
  import {
    commandLabel,
    commandShortLabel,
    commandTitle,
    isEnabled,
    pressHandlers
  } from '#lib/commands/command.ts';
  import type { IconCommand } from '#lib/commands/types.ts';
  import { Button, Tooltip } from '@immich/ui';
  import { mergeProps } from 'bits-ui';

  let {
    command,
    long = false,
    class: className = ''
  }: { command: IconCommand; long?: boolean; class?: string } = $props();

  const pressed = $derived(command.pressed?.());
  const text = $derived(long ? commandLabel(command) : commandShortLabel(command));
  const title = $derived(commandTitle(command));
</script>

{#snippet pill(props: Record<string, unknown>)}
  <Button
    size="small"
    shape="round"
    variant="ghost"
    color={pressed ? 'primary' : 'secondary'}
    class="h-9 shrink-0 gap-1 px-2.5 text-[13px] touch:h-11 {pressed
      ? 'bg-primary/15'
      : 'bg-ghost'} {className}"
    leadingIcon={command.icon}
    aria-pressed={pressed}
    disabled={!isEnabled(command)}
    {...mergeProps(props, { ...pressHandlers(command) })}
  >
    {text}
  </Button>
{/snippet}

{#if title === text}
  {@render pill({})}
{:else}
  <Tooltip text={title}>
    {#snippet child({ props })}
      {@render pill(props)}
    {/snippet}
  </Tooltip>
{/if}
