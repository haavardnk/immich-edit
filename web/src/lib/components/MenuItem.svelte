<script lang="ts">
  import { Button, Icon } from '@immich/ui';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  let {
    icon,
    label,
    keys,
    pressed,
    ...rest
  }: {
    icon: string;
    label: string;
    keys?: string;
    pressed?: boolean;
  } & Omit<HTMLButtonAttributes, 'children' | 'color' | 'size' | 'type'> = $props();
</script>

<Button
  {...rest}
  type="button"
  size="tiny"
  variant="ghost"
  color={pressed ? 'primary' : 'secondary'}
  class="h-7 w-full justify-start gap-2.5 rounded-none px-3 text-left text-xs font-normal"
  aria-label={label}
  aria-pressed={pressed}
>
  <Icon {icon} size="14px" class="shrink-0 opacity-70" aria-hidden="true" />
  <span class="min-w-0 flex-1 truncate">{label}</span>
  {#if keys}
    <span class="shrink-0 font-mono text-[10px] text-dark/65" aria-hidden="true">{keys}</span>
  {/if}
</Button>
