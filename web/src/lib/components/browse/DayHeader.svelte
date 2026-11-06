<script lang="ts">
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { held } from '#lib/input/modifiers.svelte.ts';
  import { Icon, Tooltip } from '@immich/ui';
  import { mdiCheckCircle, mdiCircleOutline } from '@mdi/js';
  import { mergeProps } from 'bits-ui';

  let {
    title,
    subtitle,
    selected,
    selectionActive,
    busy = false,
    onSelect
  }: {
    title: string;
    subtitle: string;
    selected: boolean;
    selectionActive: boolean;
    busy?: boolean;
    onSelect: (extend: boolean) => void;
  } = $props();

  const label = $derived(selected ? 'Deselect day' : 'Select day');
  const shown = $derived(selectionActive || selected);
</script>

<div
  data-testid="day-header"
  class="group/day flex h-full min-w-0 items-center gap-2 px-1 pt-1.5 text-sm font-semibold text-light-950 touch:items-end touch:gap-0 touch:pt-0 touch:pb-1.5 touch:text-[13px]"
>
  <Tooltip text={label}>
    {#snippet child({ props })}
      <button
        {...mergeProps(props, {
          type: 'button' as const,
          'aria-label': label,
          'aria-pressed': selected,
          'aria-busy': busy || undefined,
          onclick: (event: MouseEvent) => onSelect(held('extend', event)),
          class: [
            'flex h-5 shrink-0 items-center justify-center overflow-hidden rounded-full outline-none transition-[opacity,margin,width] duration-150 focus-visible:ring-2 focus-visible:ring-primary touch:order-last touch:-my-3 touch:-me-3 touch:ms-0.5 touch:size-11',
            selected ? 'text-primary' : 'text-muted hover:text-light-950',
            shown
              ? 'w-5 opacity-100'
              : '-me-2 w-0 opacity-0 group-hover/day:me-0 group-hover/day:w-5 group-hover/day:opacity-100 focus-visible:me-0 focus-visible:w-5 focus-visible:opacity-100 touch:hidden'
          ].join(' ')
        })}
      >
        <Icon icon={selected ? mdiCheckCircle : mdiCircleOutline} size="20px" aria-hidden="true" />
      </button>
    {/snippet}
  </Tooltip>
  <h2 class="shrink-0 truncate font-semibold">{title}</h2>
  <TruncationTooltip text={subtitle}>
    {#snippet child({ props })}
      <span
        {...props}
        data-truncate
        class="min-w-0 truncate text-[13px] font-normal text-dark/65 touch:ms-auto touch:ps-3 touch:text-xs"
      >
        {subtitle}
      </span>
    {/snippet}
  </TruncationTooltip>
</div>
