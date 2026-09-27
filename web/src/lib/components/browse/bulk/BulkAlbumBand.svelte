<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { Button } from '@immich/ui';
  import { addAssetsToAlbum, listAlbums, removeAssetsFromAlbum } from '$lib/api/albums';
  import SearchableSelect from '$lib/components/SearchableSelect.svelte';
  import ChosenChips from './ChosenChips.svelte';
  import { sourceId } from '$lib/assetKey';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { metadataConsent } from '$lib/stores/metadataConsent.svelte';
  import { selection } from '$lib/stores/selection.svelte';
  import { toasts } from '$lib/stores/toasts.svelte';
  import type { AlbumSummary } from '$lib/types/album';

  let { ids, busy = $bindable() }: { ids: string[]; busy: boolean } = $props();

  let albums = $state<AlbumSummary[]>([]);
  let chosen = $state<string[]>([]);

  const currentAlbum = $derived(
    page.route.id === '/albums/[id]' ? (albums.find((a) => a.id === page.params.id) ?? null) : null
  );
  const photos = $derived(`${ids.length} selected photo${ids.length === 1 ? '' : 's'}`);

  onMount(() => {
    listAlbums()
      .then((items) => {
        albums = items.sort((a, b) => a.albumName.localeCompare(b.albumName));
      })
      .catch((e: unknown) => toasts.fail('Failed to load albums', e, 6000));
  });

  async function add(): Promise<void> {
    if (busy || chosen.length === 0) return;
    if (!(await metadataConsent.gate())) return;
    busy = true;
    let failed = 0;
    for (const albumId of chosen) {
      try {
        const results = await addAssetsToAlbum(albumId, ids);
        failed += results.filter((r) => !r.success && r.error !== 'duplicate').length;
      } catch {
        failed += ids.length;
      }
    }
    busy = false;
    if (failed > 0) {
      toasts.push('warn', `${failed} album additions failed`, 6000);
      return;
    }
    toasts.push(
      'success',
      `Added ${photos} to ${chosen.length === 1 ? 'the album' : `${chosen.length} albums`}`,
      4000
    );
    chosen = [];
  }

  async function remove(album: AlbumSummary): Promise<void> {
    if (busy) return;
    if (!(await metadataConsent.gate())) return;
    busy = true;
    try {
      const results = await removeAssetsFromAlbum(album.id, ids);
      const removed = new Set(results.filter((r) => r.success).map((r) => r.id));
      browsing.assets
        .filter((a) => removed.has(sourceId(a.id)))
        .forEach((a) => browsing.remove(a.id));
      selection.clear();
      const failed = results.length - removed.size;
      if (failed > 0)
        toasts.push('warn', `${failed} could not be removed from ${album.albumName}`, 6000);
      else toasts.push('success', `Removed ${removed.size} from ${album.albumName}`, 4000);
    } catch (e) {
      toasts.fail('Failed to remove from album', e, 6000);
    } finally {
      busy = false;
    }
  }
</script>

<div class="flex flex-col gap-2 border-t px-4 py-3">
  <span class="px-1 text-xs text-dark/60">Pick albums, then add the {photos} to them</span>
  <div class="flex flex-col gap-2 sm:flex-row sm:items-end">
    <div class="min-w-0 flex-1 sm:min-w-55">
      <SearchableSelect
        options={albums}
        bind:selected={chosen}
        getId={(a) => a.id}
        getLabel={(a) => a.albumName}
        placeholder="Choose albums…"
        color="neutral"
        side="top"
      />
      <ChosenChips
        items={albums
          .filter((a) => chosen.includes(a.id))
          .map((a) => ({ id: a.id, label: a.albumName }))}
        onRemove={(id) => (chosen = chosen.filter((albumId) => albumId !== id))}
      />
    </div>
    <div class="flex gap-2">
      <Button
        size="tiny"
        color="primary"
        class="h-7 whitespace-nowrap"
        disabled={busy || chosen.length === 0}
        onclick={() => void add()}
      >
        Add to albums
      </Button>
      {#if currentAlbum}
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          class="h-7 whitespace-nowrap"
          disabled={busy}
          onclick={() => currentAlbum && void remove(currentAlbum)}
        >
          Remove from {currentAlbum.albumName}
        </Button>
      {/if}
    </div>
  </div>
</div>
