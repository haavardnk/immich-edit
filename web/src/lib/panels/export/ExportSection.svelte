<script lang="ts">
  import RevealContent from '#lib/components/RevealContent.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { Icon } from '@immich/ui';
  import { mdiChevronDown, mdiChevronRight } from '@mdi/js';
  import { Collapsible } from 'bits-ui';
  import type { Snippet } from 'svelte';

  let {
    title,
    summary = '',
    foldable = true,
    mono = false,
    children
  }: {
    title: string;
    summary?: string;
    foldable?: boolean;
    mono?: boolean;
    children: Snippet;
  } = $props();

  const id = $props.id();
  let open = $state(false);
</script>

{#if foldable}
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
            class="w-20 shrink-0 text-micro font-medium tracking-wider text-dark/65 uppercase touch:w-24 touch:text-caption"
          >
            {title}
          </span>
          <span
            data-truncate
            class="min-w-0 flex-1 truncate text-dark/80 group-hover:text-dark {mono
              ? 'font-mono'
              : ''}"
            class:invisible={open}
          >
            {summary}
          </span>
          <Icon
            icon={open ? mdiChevronDown : mdiChevronRight}
            size="14px"
            class="shrink-0 text-dark/45 group-hover:text-dark/80"
            aria-hidden="true"
          />
        </Collapsible.Trigger>
      {/snippet}
    </TruncationTooltip>
    <RevealContent class="flex flex-col gap-1 pb-1.5 touch:gap-2 touch:pb-3">
      {@render children()}
    </RevealContent>
  </Collapsible.Root>
{:else}
  <section
    aria-labelledby={id}
    class="flex flex-col gap-1 border-t border-hairline pt-1.5 touch:gap-2 touch:pt-3"
  >
    <h3
      {id}
      class="text-micro font-medium tracking-wider text-dark/65 uppercase select-none touch:text-caption"
    >
      {title}
    </h3>
    {@render children()}
  </section>
{/if}
