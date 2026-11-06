<script lang="ts">
  import { Button, Icon } from '@immich/ui';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  let {
    icon,
    label,
    iconClass = '',
    blocked = false,
    badge = 0,
    class: className = '',
    ...rest
  }: {
    icon: string;
    label: string;
    iconClass?: string;
    blocked?: boolean;
    badge?: number;
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
  <span class="relative flex">
    <Icon {icon} size="22px" class={iconClass} aria-hidden="true" />
    {#if badge > 0}
      <span
        class="absolute -top-1.5 -right-2.5 h-4 min-w-4 rounded-full bg-primary px-1 text-center text-[10px] leading-4 font-bold text-light"
        aria-hidden="true"
      >
        {badge}
      </span>
    {/if}
  </span>
  <span class="whitespace-nowrap">{label}</span>
  {#if badge > 0}
    <span class="sr-only">({badge})</span>
  {/if}
</Button>
