<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Icon } from '@immich/ui';
  import { Collapsible } from 'bits-ui';
  import { mdiChevronDown, mdiChevronRight } from '@mdi/js';

  type Props = {
    icon: string;
    label: string;
    count?: number | null;
    expanded: boolean;
    onToggle: () => void;
    children: Snippet;
  };

  let { icon, label, count, expanded, onToggle, children }: Props = $props();
</script>

<Collapsible.Root bind:open={() => expanded, () => onToggle()}>
  <Collapsible.Trigger
    class="hover:bg-subtle hover:text-primary flex w-full place-items-center gap-4 rounded-e-full py-3 ps-5 transition-[padding] delay-100 duration-100"
  >
    <Icon {icon} size="1.375em" class="shrink-0" aria-hidden="true" />
    <span class="flex-1 truncate text-left text-sm font-medium">{label}</span>
    {#if count != null && count > 0}
      <span class="text-xs text-dark/65 tabular-nums">{count}</span>
    {/if}
    <Icon
      icon={expanded ? mdiChevronDown : mdiChevronRight}
      size="1em"
      class="me-4 shrink-0 opacity-60"
      aria-hidden="true"
    />
  </Collapsible.Trigger>
  <Collapsible.Content>
    {#if expanded}
      <div class="pb-1">
        {@render children()}
      </div>
    {/if}
  </Collapsible.Content>
</Collapsible.Root>
