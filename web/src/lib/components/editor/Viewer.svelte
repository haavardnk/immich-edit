<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
</script>

<div class="flex-1 min-h-0 flex items-center justify-center bg-black/40 p-4 relative">
  {#if editor.previewUrl}
    <img
      src={editor.previewUrl}
      alt={editor.asset?.originalFileName ?? ''}
      class="max-w-full max-h-full object-contain shadow-2xl rounded"
      style="image-orientation: none;"
    />
  {:else if editor.error}
    <div class="text-red-400 text-sm">{editor.error}</div>
  {:else}
    <div class="flex gap-1">
      <div class="w-2 h-2 rounded-full bg-immich-dark-primary/50 animate-bounce" style="animation-delay: 0ms"></div>
      <div class="w-2 h-2 rounded-full bg-immich-dark-primary/50 animate-bounce" style="animation-delay: 150ms"></div>
      <div class="w-2 h-2 rounded-full bg-immich-dark-primary/50 animate-bounce" style="animation-delay: 300ms"></div>
    </div>
  {/if}
</div>
