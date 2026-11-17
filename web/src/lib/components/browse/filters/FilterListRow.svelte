<script lang="ts">
  import HighlightedText from '#lib/components/HighlightedText.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { Checkbox } from '@immich/ui';
  import type { Snippet } from 'svelte';

  let {
    label,
    count,
    query = '',
    checked,
    muted = false,
    onChange,
    leading
  }: {
    label: string;
    count?: number | null;
    query?: string;
    checked: boolean;
    muted?: boolean;
    onChange: (checked: boolean) => void;
    leading?: Snippet;
  } = $props();

  const id = $props.id();
  const touch = $derived(layout.mode === 'touch');
</script>

<div
  data-filter-row
  class="flex h-7 shrink-0 items-center gap-2 rounded-md px-1 text-label text-dark/80 hover:bg-light-950/4 touch:h-12 touch:gap-3 touch:rounded-none touch:border-b touch:border-hairline touch:px-0.5 touch:text-body touch:last:border-b-0 touch:hover:bg-transparent"
>
  <Checkbox
    {id}
    {checked}
    size={touch ? 'small' : 'tiny'}
    shape={touch ? 'round' : 'semi-round'}
    aria-label={label}
    onCheckedChange={(next) => onChange(next === true)}
  />
  <label
    for={id}
    class="flex min-w-0 flex-1 cursor-pointer items-center gap-2 select-none touch:gap-3"
  >
    {@render leading?.()}
    <TruncationTooltip text={label}>
      {#snippet child({ props })}
        <span
          {...props}
          data-truncate
          class="min-w-0 flex-1 truncate {muted ? 'text-dark/65' : 'text-light-950'}"
        >
          <HighlightedText text={label} {query} />
        </span>
      {/snippet}
    </TruncationTooltip>
    {#if count != null}
      <span class="shrink-0 text-xs text-dark/65 tabular-nums touch:text-label">{count}</span>
    {/if}
  </label>
</div>
