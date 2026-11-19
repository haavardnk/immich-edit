<script lang="ts">
  import { Button, Icon } from '@immich/ui';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  let {
    icon,
    label,
    iconClass = '',
    blocked = false,
    class: className = '',
    ...rest
  }: {
    icon: string;
    label: string;
    iconClass?: string;
    blocked?: boolean;
    class?: string;
  } & Omit<HTMLButtonAttributes, 'children' | 'color' | 'size' | 'type' | 'class'> = $props();
</script>

<Button
  {...rest}
  type="button"
  size="small"
  variant="ghost"
  color="secondary"
  aria-disabled={blocked || undefined}
  class="h-14 min-w-0 flex-1 flex-col gap-0.5 rounded-xl px-0 text-[11px] font-medium text-light-950 {blocked
    ? 'opacity-50'
    : ''} {className}"
>
  <Icon {icon} size="22px" class={iconClass} aria-hidden="true" />
  <span class="whitespace-nowrap">{label}</span>
</Button>
