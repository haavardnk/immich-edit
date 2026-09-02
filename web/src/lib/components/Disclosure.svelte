<script lang="ts">
  import { Icon } from '@immich/ui';
  import { mdiChevronDown, mdiChevronRight } from '@mdi/js';
  import { Collapsible } from 'bits-ui';
  import type { Snippet } from 'svelte';

  type Variant = 'section' | 'inline';

  let {
    open,
    title,
    onOpenChange,
    modified = false,
    variant = 'section',
    children
  }: {
    open: boolean;
    title: string;
    onOpenChange: (open: boolean) => void;
    modified?: boolean;
    variant?: Variant;
    children: Snippet;
  } = $props();

  const rootClass = $derived(variant === 'section' ? 'border-t border-hairline' : '');
  const triggerClass = $derived(
    variant === 'section'
      ? `group relative flex h-8 w-full items-center gap-2 px-3 text-[11px] font-semibold transition-colors select-none ${open ? 'bg-white/4 text-dark' : 'text-dark/65 hover:bg-white/3 hover:text-dark'}`
      : 'flex h-6 items-center gap-1 rounded px-1 text-[10px] font-medium text-dark/65 transition-colors select-none hover:bg-ghost hover:text-dark'
  );
  const iconSize = $derived(variant === 'section' ? 14 : 12);
</script>

<Collapsible.Root bind:open={() => open, onOpenChange} class={rootClass}>
  <Collapsible.Trigger class={triggerClass}>
    <Icon
      icon={open ? mdiChevronDown : mdiChevronRight}
      size={`${iconSize}px`}
      class="text-dark/35 transition-colors group-hover:text-dark/60"
      aria-hidden="true"
    />
    {title}
    {#if modified}
      <span class="ml-auto size-1.5 rounded-full bg-primary" aria-label="Modified"></span>
    {/if}
  </Collapsible.Trigger>
  <Collapsible.Content>
    {#if open}
      {@render children()}
    {/if}
  </Collapsible.Content>
</Collapsible.Root>
