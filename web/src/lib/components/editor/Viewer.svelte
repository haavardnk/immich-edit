<script lang="ts">
  let {
    src,
    pending = false,
    width = 0,
    height = 0,
    label = ''
  }: {
    src: string | null;
    pending?: boolean;
    width?: number;
    height?: number;
    label?: string;
  } = $props();
</script>

<div class="flex-1 flex flex-col items-center justify-center bg-base-100 p-4 relative">
  {#if src}
    <img
      {src}
      alt={label}
      class="max-w-full max-h-full object-contain shadow-lg"
      style="image-orientation: none;"
    />
  {:else}
    <div class="opacity-60">loading preview…</div>
  {/if}
  {#if pending}
    <div class="absolute top-2 right-2 badge badge-warning badge-sm">rendering</div>
  {/if}
  {#if width > 0 && height > 0}
    <div class="absolute bottom-2 right-2 text-xs opacity-50 font-mono">{width}×{height}</div>
  {/if}
</div>
