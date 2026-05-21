<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { isIdentity } from '$lib/types/edits';

  const neutral = $derived(isIdentity(editor.edits));
</script>

<div class="flex flex-col gap-2">
  <button
    class="btn btn-primary btn-sm"
    disabled={editor.exporting}
    onclick={() => void editor.onExport()}
  >
    {editor.exporting ? 'Exporting…' : 'Export JPEG'}
  </button>
  <button
    class="btn btn-soft btn-sm"
    disabled={neutral || editor.saving}
    onclick={() => void editor.onReset()}
  >
    Reset all
  </button>
  <div class="text-[10px] opacity-50 text-center">
    {#if editor.saving}saving…{:else if neutral}no edits{:else}edited{/if}
  </div>
</div>
