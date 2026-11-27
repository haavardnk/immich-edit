<script lang="ts">
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { Icon } from '@immich/ui';
  import { mdiChevronDown } from '@mdi/js';
  import type { Snippet } from 'svelte';

  let {
    label,
    title,
    caption,
    open = $bindable(false),
    children
  }: {
    label: string;
    title: string;
    caption: string;
    open?: boolean;
    children: Snippet;
  } = $props();

  const id = $props.id();
</script>

<section class="flex flex-col border-b border-hairline" aria-label={label}>
  <TruncationTooltip text={caption ? `${title} · ${caption}` : title}>
    {#snippet child({ props })}
      <button
        {...props}
        type="button"
        class="flex w-full items-center gap-3 py-2 ps-4 pe-3 text-left transition-colors outline-none hover:bg-light-100 focus-visible:bg-light-100 touch:min-h-13"
        aria-expanded={open}
        aria-controls={id}
        onclick={() => (open = !open)}
      >
        <span class="min-w-0 flex-1 leading-tight">
          <span data-truncate class="block truncate text-sm font-semibold text-light-950">
            {title}
          </span>
          {#if caption}
            <span data-truncate class="block truncate text-caption text-dark/65 tabular-nums">
              {caption}
            </span>
          {/if}
        </span>
        <Icon
          icon={mdiChevronDown}
          size="16px"
          class="shrink-0 text-dark/65 transition-transform {open ? 'rotate-180' : ''}"
          aria-hidden
        />
      </button>
    {/snippet}
  </TruncationTooltip>
  {#if open}
    <div {id} class="flex flex-col gap-3 px-4 pt-1 pb-3">
      {@render children()}
    </div>
  {/if}
</section>
