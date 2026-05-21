<script lang="ts">
  import { page } from '$app/state';
  import { onDestroy, untrack } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import Viewer from '$lib/components/editor/Viewer.svelte';

  const id = $derived(page.params.id as string);

  $effect(() => {
    const current = id;
    untrack(() => editor.load(current));
  });

  onDestroy(() => {
    editor.unload();
  });
</script>

{#if editor.error}
  <div class="px-4 py-2 text-xs text-red-400 bg-red-400/10 border-b border-red-400/20">{editor.error}</div>
{/if}
<Viewer />
