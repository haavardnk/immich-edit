<script lang="ts">
  import { Checkbox } from '@immich/ui';
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  let {
    checked,
    label,
    disabled = false,
    onChange,
    class: className,
    children,
    ...rest
  }: {
    checked: boolean;
    label: string;
    disabled?: boolean;
    onChange: (checked: boolean) => void;
    class?: string;
    children?: Snippet;
  } & Omit<HTMLAttributes<HTMLDivElement>, 'class'> = $props();

  const id = $props.id();
</script>

<div
  class="flex h-6 items-center gap-1.5 text-[11px] text-dark/80 {className ?? ''}"
  class:opacity-50={disabled}
  {...rest}
>
  <Checkbox {id} {checked} {disabled} size="tiny" aria-label={label} onCheckedChange={onChange} />
  <label for={id} class="select-none" class:cursor-pointer={!disabled}>
    {label}{@render children?.()}
  </label>
</div>
