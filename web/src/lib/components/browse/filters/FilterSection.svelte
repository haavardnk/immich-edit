<script lang="ts">
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { Icon } from '@immich/ui';
  import { mdiChevronDown } from '@mdi/js';
  import { Collapsible, mergeProps } from 'bits-ui';
  import type { Snippet } from 'svelte';

  let {
    open,
    title,
    summary,
    onOpenChange,
    children
  }: {
    open: boolean;
    title: string;
    summary?: string;
    onOpenChange: (open: boolean) => void;
    children: Snippet;
  } = $props();

  const summaryId = $props.id();
</script>

<Collapsible.Root bind:open={() => open, onOpenChange} class="border-b border-hairline">
  <TruncationTooltip text={summary ?? ''}>
    {#snippet child({ props })}
      <Collapsible.Trigger
        {...mergeProps(props, {
          'aria-label': title,
          'aria-describedby': summary === undefined ? undefined : summaryId
        })}
        class="group flex min-h-10 w-full items-center gap-2 py-1 pr-2.5 pl-4 text-left transition-colors outline-none select-none hover:bg-light-950/3 focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-inset data-[state=open]:bg-light-950/3 touch:min-h-13 touch:pr-1"
      >
        <span class="flex min-w-0 flex-1 flex-col">
          <span class="text-[13.5px] font-medium text-light-950 touch:text-[15.5px]">{title}</span>
          {#if summary !== undefined}
            <span
              id={summaryId}
              data-truncate
              class="truncate text-xs leading-snug text-primary touch:text-[13px]"
            >
              {summary}
            </span>
          {/if}
        </span>
        <span class="grid size-5 shrink-0 place-items-center text-dark/65 touch:size-11">
          <Icon
            icon={mdiChevronDown}
            size="18px"
            class="transition-transform group-data-[state=open]:rotate-180"
            aria-hidden="true"
          />
        </span>
      </Collapsible.Trigger>
    {/snippet}
  </TruncationTooltip>
  <Collapsible.Content>
    {#if open}
      <div class="flex flex-col gap-2 bg-light-950/3 px-4 pt-0.5 pb-3.5 touch:gap-2.5 touch:pb-4">
        {@render children()}
      </div>
    {/if}
  </Collapsible.Content>
</Collapsible.Root>
