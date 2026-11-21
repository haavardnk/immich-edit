<script lang="ts">
  import ElementTooltip from '#lib/components/ElementTooltip.svelte';
  import RevealContent from '#lib/components/RevealContent.svelte';
  import type { Snippet } from 'svelte';
  import { Icon } from '@immich/ui';
  import { Collapsible } from 'bits-ui';
  import { mdiChevronDown, mdiChevronRight } from '@mdi/js';

  type Props = {
    href: string;
    icon: string;
    label: string;
    count?: number | null;
    active: boolean;
    within: boolean;
    expanded: boolean;
    onToggle: () => void;
    children: Snippet;
  };

  let { href, icon, label, count, active, within, expanded, onToggle, children }: Props = $props();

  const current = $derived(active ? 'page' : within ? 'location' : undefined);
  const toggleLabel = $derived(`${expanded ? 'Hide' : 'Show'} ${label.toLowerCase()}`);
</script>

<Collapsible.Root bind:open={() => expanded, () => onToggle()}>
  <div
    class="flex items-center rounded-e-full transition-[padding] delay-100 duration-fast wide:rounded-xl {active ||
    (within && !expanded)
      ? 'bg-primary/10 text-primary'
      : 'hover:bg-subtle hover:text-primary'}"
  >
    <a
      {href}
      aria-current={current}
      class="flex min-w-0 flex-1 place-items-center gap-4 rounded-e-full py-3 ps-5 outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary wide:h-11 wide:rounded-xl wide:py-0 wide:ps-3"
    >
      <Icon {icon} size="1.375em" class="shrink-0" aria-hidden="true" />
      <span class="flex-1 truncate text-left text-sm font-medium wide:text-body">{label}</span>
      {#if count != null && count > 0}
        <span class="text-xs text-dark/65 tabular-nums">{count}</span>
      {/if}
    </a>
    <ElementTooltip text={toggleLabel}>
      {#snippet child({ props })}
        <Collapsible.Trigger
          {...props}
          aria-label={toggleLabel}
          class="me-2 grid size-8 shrink-0 place-items-center rounded-full text-dark/65 outline-none hover:bg-subtle hover:text-primary focus-visible:ring-2 focus-visible:ring-primary touch:size-11"
        >
          <Icon icon={expanded ? mdiChevronDown : mdiChevronRight} size="1em" aria-hidden="true" />
        </Collapsible.Trigger>
      {/snippet}
    </ElementTooltip>
  </div>
  <RevealContent class="pb-1">
    {@render children()}
  </RevealContent>
</Collapsible.Root>
