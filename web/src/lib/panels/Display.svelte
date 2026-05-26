<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { mdiRestore } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import type { TonemapKind } from '$lib/types/edits';

  function onTonemapChange(e: Event): void {
    editor.edits.output.tonemap = (e.currentTarget as HTMLSelectElement).value as TonemapKind;
    editor.onCommit();
  }

  function reset(): void {
    editor.edits.output.tonemap = 'default';
    editor.onCommit();
  }
</script>

<div class="flex flex-col gap-2.5 pb-1">
  <div class="flex items-center justify-between">
    <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Tonemap</div>
    <button
      type="button"
      class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
      title="Reset Display"
      aria-label="Reset Display"
      onclick={reset}
    >
      <Icon path={mdiRestore} size={14} />
    </button>
  </div>
  <label class="flex flex-col gap-1 text-xs">
    <span>Tonemap</span>
    <select
      class="select bg-white/5 rounded-lg text-xs h-auto py-1.5 min-h-0"
      value={editor.edits.output.tonemap}
      onchange={onTonemapChange}
    >
      <option value="default">Default</option>
      <option value="agx">AgX</option>
    </select>
  </label>
</div>
