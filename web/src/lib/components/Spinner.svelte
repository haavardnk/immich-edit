<script lang="ts">
  interface Props {
    size?: number;
    label?: string;
  }
  let { size = 20, label }: Props = $props();
</script>

<div class="inline-flex items-center gap-2 text-immich-dark-fg/60" role="status" aria-live="polite">
  <div class="flex gap-1">
    <div
      class="rounded-full bg-immich-dark-primary/50 animate-bounce"
      style="width: {size / 5}px; height: {size / 5}px; animation-delay: 0ms"
    ></div>
    <div
      class="rounded-full bg-immich-dark-primary/50 animate-bounce"
      style="width: {size / 5}px; height: {size / 5}px; animation-delay: 150ms"
    ></div>
    <div
      class="rounded-full bg-immich-dark-primary/50 animate-bounce"
      style="width: {size / 5}px; height: {size / 5}px; animation-delay: 300ms"
    ></div>
  </div>
  {#if label}
    <span class="text-xs">{label}</span>
  {/if}
</div>
