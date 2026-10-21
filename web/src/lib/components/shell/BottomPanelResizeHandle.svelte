<script lang="ts">
  import { MAX_FILMSTRIP_HEIGHT, MIN_FILMSTRIP_HEIGHT, ui } from '$lib/stores/ui.svelte';
  import ResizeHandle from './ResizeHandle.svelte';

  let { label }: { label: string } = $props();
</script>

<ResizeHandle
  {label}
  orientation="vertical"
  value={ui.filmstripHeight}
  min={MIN_FILMSTRIP_HEIGHT}
  max={MAX_FILMSTRIP_HEIGHT}
  step={8}
  shiftStep={16}
  class="absolute inset-x-0 top-0 z-20 h-2 -translate-y-1/2 cursor-row-resize outline-none focus-visible:bg-primary/30"
  onLive={ui.setFilmstripHeight}
  onCommit={ui.persistEditorUi}
/>
