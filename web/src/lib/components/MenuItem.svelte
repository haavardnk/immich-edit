<script lang="ts">
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import MenuRow from '#lib/components/MenuRow.svelte';

  let {
    icon,
    label,
    keys,
    pressed,
    ...rest
  }: {
    icon: string;
    label: string;
    keys?: string;
    pressed?: boolean;
  } & Omit<HTMLButtonAttributes, 'children' | 'color' | 'size' | 'type'> = $props();
</script>

<MenuRow {...rest} {icon} {label} {keys} {pressed} aria-label={label} />
