<script lang="ts">
  import ResetButton from './ResetButton.svelte';

  let {
    title,
    onReset,
    modified = true,
    resetTitle = `Reset ${title}`
  }: {
    title: string;
    onReset: (e: MouseEvent) => void;
    modified?: boolean;
    resetTitle?: string;
  } = $props();
</script>

<div class="flex h-6 items-center justify-between border-b border-white/6">
  <div class="text-[9px] font-semibold uppercase text-dark/65">{title}</div>
  <ResetButton title={resetTitle} disabled={!modified} onclick={onReset} />
</div>
