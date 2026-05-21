<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiExport } from '@mdi/js';
</script>

<div class="flex flex-col gap-3 px-4 pt-2">
  <button
    class="flex items-center justify-center gap-2 py-2.5 rounded-lg bg-immich-dark-primary/20 text-immich-dark-primary hover:bg-immich-dark-primary/30 text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
    disabled={editor.exporting}
    onclick={() => void editor.onExport()}
  >
    <Icon path={mdiExport} size={16} />
    {editor.exporting ? 'Exporting…' : 'Export JPEG'}
  </button>
</div>
