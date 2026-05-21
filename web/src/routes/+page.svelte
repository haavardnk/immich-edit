<script lang="ts">
  import { onMount } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { album } from '$lib/stores/album.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiImageAlbum } from '@mdi/js';

  onMount(() => {
    editor.unload();
    album.current = null;
  });
</script>

<div class="flex-1 flex flex-col items-center justify-center gap-3 text-immich-dark-fg/30">
  <Icon path={mdiImageAlbum} size={48} />
  <span class="text-sm">Pick an album from the library</span>
</div>
