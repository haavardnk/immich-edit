<script lang="ts">
  let { linked, zoom }: { linked: boolean; zoom: number | null } = $props();
</script>

<span
  role="status"
  data-testid="viewer-sync"
  class="pointer-events-none absolute top-3 right-3 z-20 flex items-center gap-1.5 rounded-full bg-black/70 px-3 py-1 text-xs font-medium text-white shadow-lg backdrop-blur-sm"
>
  <span aria-hidden="true" class="h-1.5 w-1.5 rounded-full {linked ? 'bg-success' : 'bg-warning'}"
  ></span>
  {linked ? 'Synced' : 'Not synced'}
</span>
{#if zoom !== null}
  <span
    data-testid="viewer-zoom"
    class="pointer-events-none absolute right-3 bottom-3 z-20 rounded-full bg-black/70 px-2 py-0.5 text-[11px] font-medium text-white tabular-nums"
    >{zoom}%</span
  >
{/if}
