<script lang="ts" generics="T extends string">
  import { Button } from '@immich/ui';

  let {
    options,
    value,
    label,
    onSelect
  }: {
    options: readonly { value: T; label: string }[];
    value: T;
    label: string;
    onSelect: (value: T) => void;
  } = $props();
</script>

<div
  role="group"
  aria-label={label}
  class="flex min-w-0 flex-wrap gap-0.5 rounded-2xl bg-light-100 p-0.5 text-[11px] touch:-mx-3 touch:flex-nowrap touch:overflow-x-auto touch:scrollbar-hidden touch:gap-1.5 touch:rounded-none touch:bg-transparent touch:px-3 touch:py-1.5"
>
  {#each options as option (option.value)}
    {@const selected = value === option.value}
    <Button
      type="button"
      size="tiny"
      variant={selected ? 'filled' : 'ghost'}
      color={selected ? 'primary' : 'secondary'}
      class="relative shrink-0 whitespace-nowrap touch:h-8 touch:rounded-full touch:px-3 touch:text-[13.5px] touch:font-normal touch:before:absolute touch:before:inset-x-0 touch:before:-inset-y-1.5 {selected
        ? 'touch:bg-primary/22 touch:text-light-950'
        : 'touch:bg-ghost touch:text-dark'}"
      aria-pressed={selected}
      onclick={() => onSelect(option.value)}
    >
      {option.label}
    </Button>
  {/each}
</div>
