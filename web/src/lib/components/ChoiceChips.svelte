<script lang="ts" generics="T extends string">
  import Chip from '#lib/components/chips/Chip.svelte';

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
  class="flex min-w-0 flex-wrap gap-1 touch:-mx-3 touch:flex-nowrap touch:gap-1.5 touch:overflow-x-auto touch:px-3 touch:py-1.5 touch:scrollbar-hidden"
>
  {#each options as option (option.value)}
    <Chip
      class="touch:h-8 touch:before:-inset-y-1.5"
      pressed={value === option.value}
      onPressedChange={(pressed) => {
        if (pressed) onSelect(option.value);
      }}
    >
      {option.label}
    </Chip>
  {/each}
</div>
