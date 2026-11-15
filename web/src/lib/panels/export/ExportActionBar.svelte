<script lang="ts">
  import { layout } from '#lib/device/layout.svelte.ts';
  import { Button } from '@immich/ui';
  import type { Snippet } from 'svelte';

  let {
    label,
    icon,
    busy,
    disabled,
    onStart,
    summary,
    stacked = false
  }: {
    label: string;
    icon: string;
    busy: boolean;
    disabled: boolean;
    onStart: () => void;
    summary: Snippet;
    stacked?: boolean;
  } = $props();

  const touch = $derived(layout.mode === 'touch');
</script>

<div class="flex w-full {stacked ? 'flex-col items-stretch gap-2' : 'items-center gap-3'}">
  <div class="flex min-w-0 flex-1 flex-col leading-tight">
    {@render summary()}
  </div>
  <Button
    size={touch ? 'large' : 'small'}
    color="primary"
    shape={touch ? 'round' : 'semi-round'}
    class="shrink-0 touch:h-12 touch:px-4 touch:text-[15px]"
    fullWidth={stacked}
    loading={busy}
    leadingIcon={icon}
    {disabled}
    onclick={onStart}
  >
    {label}
  </Button>
</div>
