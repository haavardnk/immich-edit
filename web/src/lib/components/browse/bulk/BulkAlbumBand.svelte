<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { Button } from '@immich/ui';
  import { listAlbums } from '#lib/api/albums.ts';
  import SearchableSelect from '#lib/components/SearchableSelect.svelte';
  import ChosenChips from './ChosenChips.svelte';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import type { AlbumSummary } from '#lib/types/album.ts';
  import { countOf } from '#lib/utils/plural.ts';

  let albums = $state<AlbumSummary[]>([]);
  let chosen = $state<string[]>([]);

  const currentAlbum = $derived(
    page.route.id === '/albums/[id]' ? (albums.find((a) => a.id === page.params.id) ?? null) : null
  );
  const photos = $derived(countOf(bulkScope.count, 'selected photo'));

  onMount(() => {
    listAlbums()
      .then((items) => {
        albums = items.sort((a, b) => a.albumName.localeCompare(b.albumName));
      })
      .catch((e: unknown) => toasts.fail('Failed to load albums', e, 6000));
  });

  async function add(): Promise<void> {
    if (await bulkActions.addToAlbums(chosen)) chosen = [];
  }
</script>

<div class="flex flex-col gap-2 border-t px-4 py-3 touch:border-t-0 touch:px-0">
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
        class="h-7 whitespace-nowrap touch:h-11"
        disabled={bulkActions.metaBusy || chosen.length === 0}
        onclick={() => void add()}
      >
        Add to albums
      </Button>
      {#if currentAlbum}
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          class="h-7 whitespace-nowrap touch:h-11"
          disabled={bulkActions.metaBusy}
          onclick={() => currentAlbum && void bulkActions.removeFromAlbum(currentAlbum)}
        >
          Remove from {currentAlbum.albumName}
        </Button>
      {/if}
    </div>
  </div>
</div>
