<script lang="ts">
  import { NavbarItem } from '@immich/ui';

  type Props = {
    href: string;
    icon: string;
    label: string;
    count?: number | null;
    active: boolean;
    hideZero?: boolean;
  };

  let { href, icon, label, count, active, hideZero = false }: Props = $props();

  const showCount = $derived(count != null && (!hideZero || count > 0));
</script>

<div class="relative">
  <NavbarItem
    title={label}
    {href}
    {icon}
    {active}
    class="wide:h-11 wide:rounded-[10px] wide:py-0 wide:ps-3 wide:[&_span]:text-[15px] {showCount
      ? 'pe-24'
      : ''}"
  />
  {#if showCount}
    <span
      class="pointer-events-none absolute right-12 top-1/2 -translate-y-1/2 text-xs text-dark/65 tabular-nums wide:right-3"
      >{count}</span
    >
  {/if}
</div>
