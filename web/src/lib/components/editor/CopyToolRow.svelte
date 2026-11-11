<script lang="ts">
  import { Checkbox } from '@immich/ui';

  let {
    label,
    state,
    depth = 0,
    strong = false,
    muted = false,
    dot = false,
    note = null,
    onChange
  }: {
    label: string;
    state: boolean | 'mixed';
    depth?: number;
    strong?: boolean;
    muted?: boolean;
    dot?: boolean;
    note?: string | null;
    onChange: (checked: boolean) => void;
  } = $props();

  const id = $props.id();
</script>

<div
  class="flex h-7 items-center gap-2.5 text-xs text-dark/80 touch:h-11 touch:gap-3 touch:border-b touch:border-hairline touch:text-[15px] {depth ===
  1
    ? 'ps-6 touch:ps-9'
    : depth === 2
      ? 'ps-12 touch:ps-18'
      : ''}"
>
  <Checkbox
    {id}
    size="tiny"
    class="border-dark/30 data-[state=checked]:border-primary data-[state=indeterminate]:border-primary data-[state=indeterminate]:[&_svg]:bg-transparent! data-[state=indeterminate]:[&_svg]:text-primary touch:size-5"
    aria-label={label}
    checked={state === true}
    indeterminate={state === 'mixed'}
    onCheckedChange={(checked) => onChange(checked)}
  />
  <label
    for={id}
    class="flex min-w-0 flex-1 cursor-pointer items-center gap-2 select-none {strong
      ? 'font-semibold text-dark'
      : muted
        ? 'text-dark/55'
        : ''}"
  >
    <span>{label}</span>
    {#if dot}
      <span class="size-1.5 shrink-0 rounded-full bg-primary"
        ><span class="sr-only">Modified</span></span
      >
    {/if}
    {#if note}
      <span class="ms-auto shrink-0 text-[11px] text-dark/50 touch:text-[13px]">{note}</span>
    {/if}
  </label>
</div>
