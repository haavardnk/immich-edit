<script lang="ts">
  import { Icon } from '@immich/ui';
  import { mdiChevronDown } from '@mdi/js';
  import { Collapsible } from 'bits-ui';
  import type { Snippet } from 'svelte';

  let {
    open,
    title,
    description,
    icon,
    onOpenChange,
    children
  }: {
    open: boolean;
    title: string;
    description: string;
    icon?: string;
    onOpenChange: (open: boolean) => void;
    children: Snippet;
  } = $props();
</script>

<Collapsible.Root
  bind:open={() => open, onOpenChange}
  class="overflow-hidden rounded-2xl border-2 border-primary/20 transition-colors"
>
  <Collapsible.Trigger
    class="group flex min-h-24 w-full items-center gap-4 px-6 py-4 text-left transition-colors select-none hover:bg-white/3"
  >
    {#if icon}
      <Icon {icon} size="24px" class="shrink-0 text-primary" aria-hidden="true" />
    {/if}
    <span class="min-w-0 flex-1">
      <span class="block text-base font-medium text-primary">{title}</span>
      <span class="mt-1 block text-sm text-gray-200">{description}</span>
    </span>
    <Icon
      icon={mdiChevronDown}
      size="20px"
      class="shrink-0 text-gray-200 transition-transform {open ? 'rotate-180' : ''}"
      aria-hidden="true"
    />
  </Collapsible.Trigger>
  <Collapsible.Content>
    {#if open}
      <div class="border-t border-primary/15 px-5 pb-5 pt-3 sm:px-6">
        {@render children()}
      </div>
    {/if}
  </Collapsible.Content>
</Collapsible.Root>
