<script lang="ts">
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { Icon } from '@immich/ui';
  import { mdiChevronDown, mdiChevronRight } from '@mdi/js';
  import { Collapsible } from 'bits-ui';
  import type { Snippet } from 'svelte';

  let {
    title,
    summary = '',
    folded = false,
    mono = false,
    children
  }: {
    title: string;
    summary?: string;
    folded?: boolean;
    mono?: boolean;
    children: Snippet;
  } = $props();

  const id = $props.id();
  let open = $derived.by(() => {
    void folded;
    return false;
  });
</script>

{#if folded}
  <Collapsible.Root
    bind:open
    role="region"
    aria-labelledby={id}
    class="flex flex-col border-t border-hairline"
  >
    <TruncationTooltip text={summary}>
      {#snippet child({ props })}
        <Collapsible.Trigger
          {...props}
          class="group flex h-9 w-full items-center gap-2 text-left text-xs outline-none select-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-inset touch:h-12 touch:text-sm"
        >
          <span
            {id}
            class="w-20 shrink-0 text-[10px] font-medium tracking-wider text-dark/65 uppercase touch:w-24 touch:text-[11px]"
          >
            {title}
          </span>
          <span
            data-truncate
            class="min-w-0 flex-1 truncate text-dark/85 group-hover:text-dark {mono
              ? 'font-mono'
              : ''}"
            class:invisible={open}
          >
            {summary}
          </span>
          <Icon
            icon={open ? mdiChevronDown : mdiChevronRight}
            size="14px"
            class="shrink-0 text-dark/45 group-hover:text-dark/70"
            aria-hidden="true"
          />
        </Collapsible.Trigger>
      {/snippet}
    </TruncationTooltip>
    <Collapsible.Content class="flex flex-col gap-1 pb-1.5">
      {#if open}
        {@render children()}
      {/if}
    </Collapsible.Content>
  </Collapsible.Root>
{:else}
  <section aria-labelledby={id} class="flex flex-col gap-1 border-t border-hairline pt-1.5">
    <h3 {id} class="text-[10px] font-medium tracking-wider text-dark/65 uppercase select-none">
      {title}
    </h3>
    {@render children()}
  </section>
{/if}
