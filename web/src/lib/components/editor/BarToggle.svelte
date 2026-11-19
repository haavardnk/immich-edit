<script lang="ts">
  import { Button } from '@immich/ui';
  import type { Snippet } from 'svelte';

  let {
    icon,
    pressed,
    label,
    title,
    disabled = false,
    onclick,
    children
  }: {
    icon: string;
    pressed: boolean;
    label: string;
    title: string;
    disabled?: boolean;
    onclick: (event: MouseEvent) => void;
    children: Snippet;
  } = $props();
</script>

<Button
  size="tiny"
  variant="ghost"
  color={pressed ? 'primary' : 'secondary'}
  class="touch:h-11 touch:px-3 {pressed
    ? 'bg-primary/10'
    : 'text-light-950/65 hover:text-light-950'}"
  leadingIcon={icon}
  {title}
  aria-label={label}
  aria-pressed={pressed}
  {disabled}
  {onclick}
>
  {@render children()}
</Button>
