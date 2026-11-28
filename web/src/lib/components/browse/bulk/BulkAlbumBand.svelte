<script lang="ts">
  import { onMount } from 'svelte';
  import { SvelteMap } from 'svelte/reactivity';
  import { listAlbums } from '#lib/api/albums.ts';
  import { assetThumbUrl } from '#lib/api/assets.ts';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { MEMBERSHIP_CAP, heldCounts } from '#lib/browse/bulk/heldMembership.ts';
  import { nextStage, type Stage } from '#lib/browse/bulk/pickStage.ts';
  import { sourceId } from '#lib/browse/assetKey.ts';
  import { viewedAlbumId } from '#lib/browse/viewedAlbum.ts';
  import { assetAlbums } from '#lib/stores/assetAlbums.svelte.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import type { AlbumSummary } from '#lib/types/album.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import BulkPickList, { type PickItem } from './BulkPickList.svelte';

  let { onDone }: { onDone?: () => void } = $props();

  let albums = $state<AlbumSummary[]>([]);
  const stages = new SvelteMap<string, Stage>();

  const sources = $derived([...new Set([...selection.selected].map(sourceId))]);
  const total = $derived(sources.length);
  const membership = $derived(heldCounts(sources, (id) => assetAlbums.albumsOf(id)));
  const viewing = $derived(viewedAlbumId());
  const items = $derived<PickItem[]>(
    albums.map((album) => ({
      id: album.id,
      label: album.albumName,
      detail: countOf(album.assetCount, 'photo'),
      thumb: album.albumThumbnailAssetId ? assetThumbUrl({ id: album.albumThumbnailAssetId }) : null
    }))
  );

  function holding(id: string): number {
    return membership?.get(id) ?? 0;
  }

  onMount(() => {
    listAlbums()
      .then((items) => {
        albums = items.sort((a, b) => a.albumName.localeCompare(b.albumName));
      })
      .catch((e: unknown) => toasts.fail('Failed to load albums', e));
  });

  $effect(() => {
    if (sources.length <= MEMBERSHIP_CAP) void assetAlbums.ensureMany(sources);
  });

  function toggle(id: string): void {
    const next = nextStage(holding(id), total, stages.get(id));
    if (next) stages.set(id, next);
    else stages.delete(id);
  }

  async function apply(): Promise<void> {
    const staged = [...stages];
    const add = staged.filter(([, stage]) => stage === 'add').map(([id]) => id);
    const remove = staged.filter(([, stage]) => stage === 'remove').map(([id]) => id);
    if (!(await bulkActions.changeAlbums(add, remove, viewing))) return;
    stages.clear();
    onDone?.();
  }
</script>

<div
  class="max-h-96 overflow-y-auto border-t border-hairline touch:flex touch:max-h-none touch:flex-1 touch:flex-col touch:overflow-visible touch:border-t-0"
>
  <BulkPickList
    {items}
    {total}
    {holding}
    {stages}
    searchLabel="Search albums"
    allTitle="All albums"
    busy={bulkActions.metaBusy}
    onToggle={toggle}
    onApply={() => void apply()}
  />
</div>
