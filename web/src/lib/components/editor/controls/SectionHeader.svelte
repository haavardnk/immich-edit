<script lang="ts">
  import type { Snippet } from 'svelte';
  import ResetButton from './ResetButton.svelte';

  let {
    title,
    onReset,
    modified = true,
    resetTitle = `Reset ${title}`,
    actions
  }: {
    title: string;
    onReset: (e: MouseEvent) => void;
    modified?: boolean;
    resetTitle?: string;
    actions?: Snippet;
  } = $props();
</script>

<div class="flex h-6 items-center justify-between border-b border-white/6">
  <div class="text-[9px] font-semibold uppercase text-dark/65">{title}</div>
  <div class="flex items-center gap-0.5">
    {@render actions?.()}
    <ResetButton title={resetTitle} disabled={!modified} onclick={onReset} />
  </div>
</div>
