<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
</script>

<div class="flex-1 min-h-0 flex items-center justify-center bg-base-100 p-4 relative">
  {#if editor.previewUrl}
    <img
      src={editor.previewUrl}
      alt={editor.asset?.originalFileName ?? ''}
      class="max-w-full max-h-full object-contain shadow-2xl"
      style="image-orientation: none;"
    />
  {:else if editor.error}
    <div class="text-error text-sm">{editor.error}</div>
  {:else}
    <span class="loading loading-dots loading-md opacity-50"></span>
  {/if}
</div>
