<script lang="ts">
  import { untrack } from 'svelte';
  import InfoSection from '#lib/components/browse/info/InfoSection.svelte';
  import AddChip from '#lib/components/AddChip.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import SearchableSelect from '#lib/components/SearchableSelect.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { thumbUrl } from '#lib/api/assets.ts';
  import { addToAlbums } from '#lib/browse/bulk/bulkAlbums.ts';
  import { assetAlbums } from '#lib/stores/assetAlbums.svelte.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiImageAlbum } from '@mdi/js';

  let { asset }: { asset: AssetSummary } = $props();

  let picking = $state(false);

  const albums = $derived(assetAlbums.albumsOf(asset.id));
  const failure = $derived(assetAlbums.errorOf(asset.id));
  const memberIds = $derived((albums ?? []).map((album) => album.id));

  $effect(() => {
    const id = asset.id;
    if (albums === null && failure === null) untrack(() => void assetAlbums.ensure(id));
  });

  function setPicking(open: boolean): void {
    picking = open;
    if (open) library.ensure('albums').catch((e: unknown) => toasts.fail('albums', e));
  }

  function pick(next: string[]): void {
    const current = new Set(memberIds);
    const added = next.filter((id) => !current.has(id));
    if (added.length === 0) return;
    picking = false;
    void addToAlbums([asset.id], added, 'photo');
  }
</script>

<InfoSection title="Albums">
  {#snippet action()}
    <Popover
      open={picking}
      anchor="bottom"
      align="end"
      onOpenChange={setPicking}
      appearance="panel"
      contentClass="w-72 overflow-visible! border-0! bg-transparent! p-0 shadow-none! ring-0!"
      sheetClass="flex flex-col"
      sheetPicker
      title="Add to album"
    >
      {#snippet trigger(popoverProps)}
        <AddChip aria-label="Add to album" {...popoverProps} />
      {/snippet}
      <SearchableSelect
        options={library.albums}
        selected={memberIds}
        getId={(album) => album.id}
        getLabel={(album) => album.albumName}
        placeholder="Add to album…"
        compact
        color="neutral"
        side="top"
        autofocus
        closeOnSelect={false}
        onSelectedChange={pick}
      />
    </Popover>
  {/snippet}
  {#if failure}
    <div class="flex items-center justify-between gap-2 text-xs text-light-950/60 touch:text-sm">
      <span>Albums did not load</span>
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="touch:h-11"
        onclick={() => void assetAlbums.retry(asset.id)}
      >
        Retry
      </Button>
    </div>
  {:else if albums === null}
    <span class="text-xs text-light-950/60 touch:text-sm" role="status">Loading albums…</span>
  {:else if albums.length === 0}
    <span class="text-xs text-light-950/60 touch:text-sm">Not in any album</span>
  {:else}
    <ul class="flex flex-col gap-1.5" aria-label="Albums with this photo">
      {#each albums as album (album.id)}
        <li class="flex min-w-0 items-center gap-2.5 touch:min-h-11">
          {#if album.albumThumbnailAssetId}
            <img
              src={thumbUrl(album.albumThumbnailAssetId)}
              alt=""
              loading="lazy"
              class="size-7 shrink-0 rounded object-cover touch:size-9"
            />
          {:else}
            <span
              class="grid size-7 shrink-0 place-items-center rounded bg-light-950/8 text-light-950/60 touch:size-9"
            >
              <Icon icon={mdiImageAlbum} size="16px" aria-hidden="true" />
            </span>
          {/if}
          <TruncationTooltip text={album.albumName}>
            {#snippet child({ props })}
              <span
                {...props}
                class="min-w-0 flex-1 truncate text-xs font-medium text-light-950/90 touch:text-[15px]"
              >
                {album.albumName}
              </span>
            {/snippet}
          </TruncationTooltip>
          <span class="shrink-0 text-[11px] text-light-950/60 tabular-nums touch:text-sm">
            <span aria-hidden="true">{album.assetCount}</span>
            <span class="sr-only">{countOf(album.assetCount, 'photo')}</span>
          </span>
        </li>
      {/each}
    </ul>
  {/if}
</InfoSection>
