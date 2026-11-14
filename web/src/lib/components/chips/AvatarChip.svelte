<script lang="ts">
  import HighlightedText from '#lib/components/HighlightedText.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { Icon } from '@immich/ui';
  import { mdiCheck } from '@mdi/js';
  import { Toggle } from 'bits-ui';

  let {
    name,
    image,
    icon,
    query = '',
    pressed,
    onPressedChange
  }: {
    name: string;
    image?: string;
    icon?: string;
    query?: string;
    pressed: boolean;
    onPressedChange: (pressed: boolean) => void;
  } = $props();

  const dashed = $derived(image === undefined);
</script>

<TruncationTooltip text={name}>
  {#snippet child({ props })}
    <Toggle.Root
      {...props}
      bind:pressed={() => pressed, onPressedChange}
      aria-label={name}
      class="group flex w-14 shrink-0 flex-col items-center gap-1 rounded-lg text-center text-[11px] leading-tight outline-none focus-visible:ring-2 focus-visible:ring-primary touch:w-16.5 touch:text-xs {dashed
        ? 'text-dark/65'
        : 'text-dark/85'}"
    >
      <span
        class="relative grid size-10 place-items-center rounded-full ring-offset-light transition-shadow group-aria-pressed:ring-offset-2 touch:size-13 {dashed
          ? 'border border-dashed border-light-950/25 group-aria-pressed:border-solid group-aria-pressed:border-primary group-aria-pressed:bg-primary/20'
          : 'ring-1 ring-hairline group-aria-pressed:ring-2 group-aria-pressed:ring-primary'}"
      >
        {#if image !== undefined}
          <img
            src={image}
            alt=""
            loading="lazy"
            class="no-callout size-full rounded-full bg-light-200 object-cover"
          />
        {:else if icon}
          <Icon {icon} size="45%" aria-hidden="true" />
        {/if}
        {#if pressed}
          <span
            class="absolute -right-0.5 -bottom-0.5 grid size-4 place-items-center rounded-full bg-primary text-light ring-2 ring-light touch:size-5"
            aria-hidden="true"
          >
            <Icon icon={mdiCheck} size="75%" />
          </span>
        {/if}
      </span>
      <span data-truncate class="w-full truncate"><HighlightedText text={name} {query} /></span>
    </Toggle.Root>
  {/snippet}
</TruncationTooltip>
