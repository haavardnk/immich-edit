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
    summary
  }: {
    label: string;
    icon: string;
    busy: boolean;
    disabled: boolean;
    onStart: () => void;
    summary: Snippet;
  } = $props();

  const touch = $derived(layout.mode === 'touch');
</script>

<div class="flex w-full items-center gap-3">
  <div class="flex min-w-0 flex-1 flex-col leading-tight">
    {@render summary()}
  </div>
  <Button
    size={touch ? 'large' : 'small'}
    color="primary"
    shape={touch ? 'round' : 'semi-round'}
    class="shrink-0 touch:h-12 touch:px-4 touch:text-[15px]"
    loading={busy}
    leadingIcon={icon}
    {disabled}
    onclick={onStart}
  >
    {label}
  </Button>
</div>
