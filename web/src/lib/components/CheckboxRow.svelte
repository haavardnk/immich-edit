<script lang="ts">
  import { Checkbox } from '@immich/ui';
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  let {
    checked,
    label,
    disabled = false,
    look = 'row',
    onChange,
    class: className,
    children,
    ...rest
  }: {
    checked: boolean;
    label: string;
    disabled?: boolean;
    look?: 'row' | 'chip';
    onChange: (checked: boolean) => void;
    class?: string;
    children?: Snippet;
  } & Omit<HTMLAttributes<HTMLDivElement>, 'class'> = $props();

  const id = $props.id();
  const looks = {
    row: 'touch:h-11 touch:gap-2.5 touch:text-label',
    chip: 'touch:h-9 touch:gap-2 touch:rounded-full touch:bg-ghost touch:px-3.5 touch:text-sm touch:font-medium touch:text-dark'
  };
</script>

<div
  class="flex h-6 items-center gap-1.5 text-caption text-dark/80 {looks[look]} {className ?? ''}"
  class:opacity-50={disabled}
  {...rest}
>
  <Checkbox {id} {checked} {disabled} size="tiny" aria-label={label} onCheckedChange={onChange} />
  <label for={id} class="select-none" class:cursor-pointer={!disabled}>
    {label}{@render children?.()}
  </label>
</div>
