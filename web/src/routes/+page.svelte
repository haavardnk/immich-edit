<script lang="ts">
  import { onMount } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { album } from '$lib/stores/album.svelte';

  onMount(() => {
    editor.unload();
    album.current = null;
  });
</script>

<div class="flex-1 flex items-center justify-center text-sm opacity-50">
  Pick an album from the library
</div>
