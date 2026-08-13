<script lang="ts">
  import ResetButton from './ResetButton.svelte';

  let {
    title,
    onReset,
    resetTitle = `Reset ${title}`
  }: {
    title: string;
    onReset: (e: MouseEvent) => void;
    resetTitle?: string;
  } = $props();
</script>

<div class="flex items-center justify-between">
  <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">{title}</div>
  <ResetButton title={resetTitle} onclick={onReset} />
</div>
