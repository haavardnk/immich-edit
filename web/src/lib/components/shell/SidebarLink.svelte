<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';

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

<a
  {href}
  class="flex items-center gap-2.5 py-2 px-4 transition-colors {active
    ? 'bg-immich-dark-primary/15 text-immich-dark-primary'
    : 'text-immich-dark-fg/70 hover:bg-white/5'}"
>
  <Icon path={icon} size={18} class="flex-none" />
  <span class="text-[13px] font-medium flex-1">{label}</span>
  {#if showCount}
    <span class="text-[11px] text-immich-dark-fg/30 tabular-nums">{count}</span>
  {/if}
  <span class="w-4"></span>
</a>
