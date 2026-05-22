<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiChevronDown, mdiChevronRight } from '@mdi/js';

  type Props = {
    icon: string;
    label: string;
    count?: number | null;
    expanded: boolean;
    onToggle: () => void;
    children: Snippet;
  };

  let { icon, label, count, expanded, onToggle, children }: Props = $props();
</script>

<button
  class="w-full flex items-center gap-2.5 py-2 px-4 transition-colors text-immich-dark-fg/70 hover:bg-white/5"
  onclick={onToggle}
>
  <Icon path={icon} size={18} class="flex-none" />
  <span class="text-[13px] font-medium flex-1 text-left">{label}</span>
  {#if count != null && count > 0}
    <span class="text-[11px] text-immich-dark-fg/30 tabular-nums">{count}</span>
  {/if}
  <Icon path={expanded ? mdiChevronDown : mdiChevronRight} size={16} class="opacity-40" />
</button>
{#if expanded}
  <div class="pb-1">
    {@render children()}
  </div>
{/if}
