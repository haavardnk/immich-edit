<script lang="ts">
  import HighlightedText from '#lib/components/HighlightedText.svelte';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
  import { Icon } from '@immich/ui';
  import { mdiChevronRight } from '@mdi/js';
  import type { HTMLAnchorAttributes } from 'svelte/elements';

  let {
    icon,
    label,
    query = '',
    count = null,
    last = false,
    ...rest
  }: {
    icon: string;
    label: string;
    query?: string;
    count?: number | null;
    last?: boolean;
  } & Omit<HTMLAnchorAttributes, 'children'> = $props();
</script>

<TooltipLink
  {...rest}
  tooltip={label}
  class="no-callout flex min-h-11 items-center gap-3.5 ps-4 pe-2.5 outline-none hover:bg-subtle focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary sm:ps-5 touch:min-h-14.5"
>
  <span
    class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-light-200 text-primary touch:size-9.5 touch:rounded-[10px]"
  >
    <Icon {icon} size="18px" class="touch:size-5.5" aria-hidden="true" />
  </span>
  <span
    class="flex min-h-11 min-w-0 flex-1 items-center gap-2 self-stretch touch:min-h-14.5 {last
      ? ''
      : 'border-b border-hairline'}"
  >
    <span data-truncate class="min-w-0 flex-1 truncate text-sm text-light-950 touch:text-base">
      <HighlightedText text={label} {query} />
    </span>
    {#if count !== null}
      <span class="shrink-0 text-xs text-dark/65 tabular-nums touch:text-sm">{count}</span>
    {/if}
    <Icon icon={mdiChevronRight} size="18px" class="shrink-0 text-dark/65" aria-hidden="true" />
  </span>
</TooltipLink>
