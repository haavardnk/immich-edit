<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { isIdentity } from '$lib/types/edits';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiExport, mdiRestore } from '@mdi/js';

  const neutral = $derived(isIdentity(editor.edits));
</script>

<div class="flex flex-col gap-2">
  <button
    class="flex items-center justify-center gap-2 py-2 rounded-lg bg-immich-dark-primary/20 text-immich-dark-primary hover:bg-immich-dark-primary/30 text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
    disabled={editor.exporting}
    onclick={() => void editor.onExport()}
  >
    <Icon path={mdiExport} size={16} />
    {editor.exporting ? 'Exporting…' : 'Export JPEG'}
  </button>
  <button
    class="flex items-center justify-center gap-2 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors disabled:opacity-20 disabled:cursor-not-allowed"
    disabled={neutral || editor.saving}
    onclick={() => void editor.onReset()}
  >
    <Icon path={mdiRestore} size={16} />
    Reset all
  </button>
  <div class="text-[10px] text-immich-dark-fg/30 text-center">
    {#if editor.saving}saving…{:else if neutral}no edits{:else}edited{/if}
  </div>
</div>
