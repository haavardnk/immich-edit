<script lang="ts">
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { reveal } from '#lib/motion/reveal.ts';
  import { Icon } from '@immich/ui';
  import { mdiChevronDown } from '@mdi/js';
  import type { Snippet } from 'svelte';

  let {
    label,
    meta = '',
    value,
    open,
    onToggle,
    trailing,
    children
  }: {
    label: string;
    meta?: string;
    value: string;
    open: boolean;
    onToggle: () => void;
    trailing?: Snippet;
    children: Snippet;
  } = $props();

  const id = $props.id();
  const changed = $derived(value !== '');
  const shown = $derived(changed ? value : 'No change');
  const parts = $derived([label, meta, shown].filter(Boolean));
</script>

<div
  class="flex flex-col rounded-lg transition-[background-color,box-shadow] duration-base {open
    ? 'bg-light-100 ring-1 ring-hairline'
    : ''}"
  role="group"
  aria-label={label}
>
  <div class="flex items-center gap-1 pe-1">
    <TruncationTooltip text={parts.join(' · ')}>
      {#snippet child({ props })}
        <button
          {...props}
          type="button"
          class="flex min-h-8 min-w-0 flex-1 items-center gap-2 rounded-lg py-1.5 ps-3 pe-2 text-left text-sm transition-colors outline-none hover:bg-light-100 focus-visible:bg-light-100 touch:min-h-11"
          aria-expanded={open}
          aria-controls={id}
          aria-label={parts.join(', ')}
          onclick={onToggle}
        >
          <span data-truncate class="min-w-0 truncate font-semibold text-light-950">{label}</span>
          {#if meta}
            <span class="shrink-0 text-caption text-dark/65 tabular-nums">{meta}</span>
          {/if}
          <span
            data-truncate
            class="ms-auto min-w-0 truncate text-end tabular-nums {changed
              ? 'font-semibold text-primary'
              : 'text-caption text-dark/65'}"
          >
            {shown}
          </span>
          <Icon
            icon={mdiChevronDown}
            size="16px"
            class="shrink-0 text-dark/65 transition-transform duration-base {open
              ? 'rotate-180'
              : ''}"
            aria-hidden
          />
        </button>
      {/snippet}
    </TruncationTooltip>
    {@render trailing?.()}
  </div>
  {#if open}
    <div {id} class="flex flex-col gap-3 px-3 pt-1 pb-3" transition:reveal>
      {@render children()}
    </div>
  {/if}
</div>
