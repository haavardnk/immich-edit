<script lang="ts">
  import type { TagCheck } from '#lib/browse/tagTree.ts';
  import { Checkbox } from '@immich/ui';
  import type { Snippet } from 'svelte';

  let {
    check,
    disabled = false,
    depth = 0,
    onChange,
    lead,
    children
  }: {
    check: TagCheck;
    disabled?: boolean;
    depth?: number;
    onChange: (on: boolean) => void;
    lead?: Snippet;
    children: Snippet;
  } = $props();

  const id = $props.id();
</script>

<li
  class="flex min-h-7 items-center gap-1 pe-3 text-xs touch:min-h-11 touch:border-b touch:border-hairline touch:pe-4 touch:text-[15px]"
  style:padding-inline-start="{12 + depth * 16}px"
>
  {@render lead?.()}
  <label
    for={id}
    class="flex min-w-0 flex-1 items-center gap-1.5 {disabled ? '' : 'cursor-pointer'}"
  >
    {@render children()}
  </label>
  <Checkbox
    {id}
    size="tiny"
    class="border-dark/30 data-[state=checked]:border-primary data-[state=indeterminate]:border-primary data-[state=indeterminate]:[&_svg]:bg-transparent! data-[state=indeterminate]:[&_svg]:text-primary touch:size-5"
    checked={check === 'checked'}
    indeterminate={check === 'mixed'}
    {disabled}
    onCheckedChange={(on) => onChange(on)}
  />
</li>
