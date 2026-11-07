<script lang="ts">
  import { commandTitle, isEnabled, pressHandlers } from '#lib/commands/command.ts';
  import type { IconCommand } from '#lib/commands/types.ts';
  import { IconButton, type Color, type Shape, type Size, type Variants } from '@immich/ui';

  type Props = {
    command: IconCommand;
    size?: Size;
    variant?: Variants;
    color?: Color;
    shape?: Shape;
    class?: string;
    before?: () => void;
  };

  let { command, color = 'secondary', before, ...rest }: Props = $props();

  const pressed = $derived(command.pressed?.());
  const item = $derived({
    icon: command.icon,
    title: commandTitle(command),
    'aria-label': commandTitle(command),
    'aria-pressed': pressed,
    color: pressed ? 'primary' : color,
    disabled: !isEnabled(command)
  } as const);
</script>

<IconButton {...rest} {...item} {...pressHandlers(command, before)} />
