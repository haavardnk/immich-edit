<script lang="ts">
  import { page } from '$app/state';
  import { onMount, untrack } from 'svelte';
  import { folderAssets } from '$lib/api/folders';
  import { editor } from '$lib/stores/editor.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import type { AssetSummary } from '$lib/types/album';

  let assets = $state<AssetSummary[]>([]);
  let loading = $state(false);
  let folderPath = $state('');

  const queryPath = $derived(page.url.searchParams.get('path') ?? '');

  async function loadFolder(path: string): Promise<void> {
    if (!path) return;
    folderPath = path;
    loading = true;
    const raw = await folderAssets(path);
    assets = raw.map((a) => ({
      id: a.id,
      originalFileName: a.originalFileName,
      type: a.type,
      fileCreatedAt: a.fileCreatedAt,
      updatedAt: a.updatedAt,
      checksum: a.checksum,
    }));
    browsing.set(assets);
    loading = false;
  }

  $effect(() => {
    const p = queryPath;
    untrack(() => loadFolder(p));
  });

  onMount(() => {
    editor.unload();
  });
</script>

{#if loading}
  <div class="flex-1 flex items-center justify-center text-sm text-immich-dark-fg/40">loading…</div>
{:else if !folderPath}
  <div class="flex-1 flex items-center justify-center text-sm text-immich-dark-fg/30">Select a folder</div>
{:else}
  <div class="px-4 py-2.5 text-xs text-immich-dark-fg/40 border-b border-white/5 flex items-center gap-2">
    <span class="font-semibold text-immich-dark-fg/70 text-sm truncate">{folderPath}</span>
    <span class="text-immich-dark-fg/20">·</span>
    <span class="flex-none">{assets.length} assets</span>
  </div>
  <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
    <AssetGrid {assets} />
  </div>
{/if}
