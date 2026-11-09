<script lang="ts">
  import { Button, Icon } from '@immich/ui';
  import { mdiChevronDown, mdiChevronRight, mdiRestore } from '@mdi/js';
  import { Collapsible } from 'bits-ui';
  import type { Snippet } from 'svelte';

  type Variant = 'section' | 'inline';

  let {
    open,
    title,
    onOpenChange,
    modified = false,
    onReset,
    variant = 'section',
    children
  }: {
    open: boolean;
    title: string;
    onOpenChange: (open: boolean) => void;
    modified?: boolean;
    onReset?: () => void;
    variant?: Variant;
    children: Snippet;
  } = $props();

  const rootClass = $derived(variant === 'section' ? 'border-t border-hairline' : '');
  const resetShown = $derived(modified && onReset !== undefined);
  const triggerClass = $derived(
    variant === 'section'
      ? `group relative flex w-full items-center gap-2 px-3 text-[11px] font-semibold transition-colors select-none touch:h-14 touch:gap-3 touch:px-4 touch:text-[15.5px] touch:font-medium touch:text-light-950 h-8 ${resetShown ? 'pr-8 touch:pr-14' : ''} ${open ? 'bg-light-950/4 text-dark' : 'text-dark/65 hover:bg-light-950/3 hover:text-dark'}`
      : 'flex h-6 items-center gap-1 rounded px-1 text-[10px] font-medium text-dark/65 transition-colors select-none hover:bg-ghost hover:text-dark'
  );
  const iconSize = $derived(variant === 'section' ? 14 : 12);
</script>

<Collapsible.Root bind:open={() => open, onOpenChange} class="relative {rootClass}">
  <Collapsible.Trigger class={triggerClass}>
    <Icon
      icon={open ? mdiChevronDown : mdiChevronRight}
      size={`${iconSize}px`}
      class="text-dark/35 transition-colors group-hover:text-dark/60"
      aria-hidden="true"
    />
    {title}
    {#if modified && !onReset}
      <span class="ml-auto size-1.5 rounded-full bg-primary" aria-label="Modified"></span>
    {/if}
  </Collapsible.Trigger>
  {#if resetShown}
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      class="group/reset absolute top-1 right-0.75 size-6 p-0 touch:top-1.5 touch:right-1 touch:size-11"
      title="Reset {title} panel"
      aria-label="Reset {title} panel"
      onclick={onReset}
    >
      <span
        class="size-1.5 rounded-full bg-primary group-hover/reset:hidden group-focus-visible/reset:hidden touch:hidden"
        aria-hidden="true"
      ></span>
      <Icon
        icon={mdiRestore}
        size="60%"
        class="hidden group-hover/reset:block group-focus-visible/reset:block touch:block touch:size-5.5"
        aria-hidden="true"
      />
    </Button>
  {/if}
  <Collapsible.Content>
    {#if open}
      {@render children()}
    {/if}
  </Collapsible.Content>
</Collapsible.Root>
