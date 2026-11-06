<script lang="ts">
  import type { AssetSummary } from '#lib/types/album.ts';
  import { page } from '$app/state';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { editorHref } from '#lib/editor/navigation.ts';
  import { virtualCopies } from '#lib/stores/virtualCopies.svelte.ts';
  import { isRejected } from '#lib/browse/reject.ts';
  import * as bulkMeta from '#lib/browse/bulkMeta.ts';
  import { NOT_OWNER, SOME_NOT_OWNER, ownsAsset } from '#lib/browse/ownership.ts';
  import { NOT_OWNER_TRASH } from '#lib/browse/trash.ts';
  import { isCopy } from '#lib/browse/assetKey.ts';
  import { trash } from '#lib/stores/trash.svelte.ts';
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import IconLink from '#lib/components/IconLink.svelte';
  import { labelOf, type LabelColor } from '#lib/stores/labels.ts';
  import LabelPicker from '#lib/components/LabelPicker.svelte';
  import FavoriteButton from '#lib/components/FavoriteButton.svelte';
  import RejectButton from '#lib/components/RejectButton.svelte';
  import StarRating from '#lib/components/StarRating.svelte';
  import { commonValue, coverage } from '#lib/utils/selectionState.ts';
  import BulkActionsDialog from './BulkActionsDialog.svelte';
  import BulkTagBand from './BulkTagBand.svelte';
  import BulkAlbumBand from './BulkAlbumBand.svelte';
  import {
    Button,
    ControlBar,
    ControlBarContent,
    ControlBarHeader,
    ControlBarOverflow,
    IconButton
  } from '@immich/ui';
  import type { MultiMode } from '#lib/browse/compareEntry.ts';
  import { MAX_PANES } from '#lib/stores/compare.svelte.ts';
  import {
    mdiClose,
    mdiImageAlbum,
    mdiCompare,
    mdiContentDuplicate,
    mdiImageEditOutline,
    mdiTagOutline,
    mdiTrashCanOutline,
    mdiTuneVariant,
    mdiViewGridOutline
  } from '@mdi/js';

  let {
    assets,
    selectedIds,
    onClear,
    onMulti,
    onSelectAll,
    hasMore = false,
    loadingMore = false,
    selectingAll = false
  }: {
    assets: AssetSummary[];
    selectedIds: string[];
    onClear: () => void;
    onMulti: (mode: MultiMode) => void;
    onSelectAll: () => Promise<boolean>;
    hasMore?: boolean;
    loadingMore?: boolean;
    selectingAll?: boolean;
  } = $props();

  let busy = $state(false);
  let band = $state<'tags' | 'albums' | null>(null);
  let bulkActionsOpen = $state(false);
  let bar = $state<HTMLDivElement | null>(null);

  export function focusFirst(): void {
    bar?.querySelector<HTMLElement>('button:not([disabled]), a[href]')?.focus();
  }

  let metaBusy = $derived(busy || selectingAll || virtualCopies.busy);
  let count = $derived(selectedIds.length);
  let targetCount = $derived(selectingAll ? assets.length : count);
  let picked = $derived.by(() => {
    const ids = new Set(selectedIds);
    return assets.filter((a) => ids.has(a.id));
  });
  let single = $derived(!selectingAll && count === 1 ? (picked[0] ?? null) : null);
  let allOwned = $derived(picked.every(ownsAsset));
  let trashable = $derived(picked.some((a) => isCopy(a.id) || ownsAsset(a)));
  let notOwnedReason = $derived(count === 1 ? NOT_OWNER : SOME_NOT_OWNER);
  let multiMode = $derived<MultiMode>(count === 2 ? 'compare' : 'survey');
  let canMulti = $derived(!selectingAll && count >= 2 && count <= MAX_PANES);
  let multiLabel = $derived(multiMode === 'compare' ? 'Compare selected' : 'Survey selected');
  let multiTitle = $derived(
    canMulti
      ? hint(multiLabel, multiMode === 'compare' ? 'enterCompare' : 'enterSurvey')
      : `Survey takes up to ${MAX_PANES} photos`
  );
  let photos = $derived(targetCount === 1 ? 'this photo' : 'selected photos');
  let favoriteState = $derived(coverage(picked, count, (a) => a.isFavorite));
  let rejectState = $derived(coverage(picked, count, (a) => isRejected(a)));
  let commonRating = $derived(commonValue(picked, count, (a) => a.exifInfo?.rating ?? 0));
  let commonLabel = $derived(commonValue(picked, count, (a) => labelOf(a)));
  let showSelectAll = $derived.by(() => {
    if (hasMore) return true;
    const picked = new Set(selectedIds);
    return assets.some((asset) => !picked.has(asset.id));
  });

  async function selectAll(): Promise<void> {
    if (busy || loadingMore || selectingAll) return;
    band = null;
    bulkActionsOpen = false;
    await onSelectAll();
  }

  async function runBulk(write: (ids: string[]) => Promise<unknown>): Promise<void> {
    if (metaBusy) return;
    busy = true;
    try {
      await write([...selectedIds]);
    } finally {
      busy = false;
    }
  }

  function setFavorite(value: boolean): Promise<void> {
    return runBulk((ids) => bulkMeta.setFavorite(ids, value));
  }

  function createCopies(): Promise<void> {
    return runBulk(virtualCopies.create);
  }

  function setRating(value: number | null): Promise<void> {
    return runBulk((ids) => bulkMeta.setRating(ids, value));
  }

  function applyReject(value: boolean): Promise<void> {
    return runBulk((ids) => bulkMeta.setRejected(ids, value));
  }

  function applyLabel(color: LabelColor | null): Promise<void> {
    return runBulk((ids) => bulkMeta.setLabel(ids, color));
  }
</script>

{#if count > 0 || selectingAll}
  <div
    bind:this={bar}
    role="toolbar"
    aria-label="Selection actions"
    class="absolute bottom-[calc(1rem+var(--chrome-inset,0px))] left-1/2 z-40 flex w-max max-w-[calc(100%-2rem)] -translate-x-1/2 flex-col overflow-hidden rounded-lg bg-light-100 shadow-2xl"
  >
    <ControlBar static shape="rectangle" class="min-w-0 px-3">
      <ControlBarHeader class="flex items-center gap-2 p-1">
        <span class="min-w-0 whitespace-nowrap text-sm font-medium" aria-live="polite">
          {#if single}
            <span class="sr-only">1 selected</span>
            <span class="block max-w-64 truncate" aria-hidden="true" title={single.originalFileName}
              >{single.originalFileName}</span
            >
          {:else}
            {targetCount} selected
          {/if}
        </span>
        {#if showSelectAll}
          <Button
            size="tiny"
            variant="ghost"
            color="secondary"
            title="Load and select all photos"
            loading={selectingAll}
            disabled={busy || loadingMore}
            onclick={selectAll}
          >
            Select all
          </Button>
        {/if}
      </ControlBarHeader>

      <ControlBarContent
        class="-my-1 min-w-0 gap-1 overflow-x-auto overscroll-contain p-1 scrollbar-hidden"
      >
        <span class="mx-1 h-5 w-px shrink-0 bg-dark/10" aria-hidden="true"></span>
        {#if single}
          <IconLink
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiImageEditOutline}
            tooltip={hint('Open in editor', 'openEditor')}
            aria-label="Open in editor"
            href={editorHref(single.id, `${page.url.pathname}${page.url.search}`)}
          />
        {:else}
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={multiMode === 'compare' ? mdiCompare : mdiViewGridOutline}
            title={multiTitle}
            aria-label={multiLabel}
            disabled={!canMulti}
            onclick={() => onMulti(multiMode)}
          />
        {/if}
        <span class="mx-1 h-5 w-px shrink-0 bg-dark/10" aria-hidden="true"></span>

        <DisabledReason
          locked={!allOwned}
          reason={notOwnedReason}
          class="flex shrink-0 items-center gap-1"
        >
          <FavoriteButton
            size="medium"
            isFavorite={favoriteState === 'all'}
            mixed={favoriteState === 'some'}
            disabled={metaBusy || !allOwned}
            ontoggle={() => void setFavorite(favoriteState !== 'all')}
          />
          <StarRating
            size={20}
            rating={commonRating ?? 0}
            mixed={commonRating === null}
            disabled={metaBusy || !allOwned}
            onchange={(value) => void setRating(value)}
          />
          <RejectButton
            size="medium"
            isRejected={rejectState === 'all'}
            mixed={rejectState === 'some'}
            disabled={metaBusy || !allOwned}
            ontoggle={() => void applyReject(rejectState !== 'all')}
          />
          <LabelPicker
            size="medium"
            label={commonLabel}
            disabled={metaBusy || !allOwned}
            onchange={(color) => void applyLabel(color)}
          />
        </DisabledReason>
        <span class="mx-1 h-5 w-px shrink-0 bg-dark/10" aria-hidden="true"></span>
        <div class="flex shrink-0 items-center gap-1">
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiContentDuplicate}
            title={hint(
              targetCount === 1 ? 'Create a virtual copy' : 'Create a virtual copy of each',
              'createVirtualCopy'
            )}
            aria-label="Create virtual copy"
            disabled={metaBusy}
            onclick={() => void createCopies()}
          />
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiTuneVariant}
            title={`Paste edits, apply a preset or export ${photos}`}
            aria-label="Edit and export selected"
            disabled={selectingAll}
            onclick={() => (bulkActionsOpen = true)}
          />
          <DisabledReason locked={!allOwned} reason={notOwnedReason} class="flex">
            <IconButton
              size="medium"
              variant={band === 'tags' ? 'filled' : 'ghost'}
              color={band === 'tags' ? 'primary' : 'secondary'}
              icon={mdiTagOutline}
              title="Tags"
              aria-label="Tags"
              aria-pressed={band === 'tags'}
              disabled={metaBusy || !allOwned}
              onclick={() => (band = band === 'tags' ? null : 'tags')}
            />
          </DisabledReason>
          <IconButton
            size="medium"
            variant={band === 'albums' ? 'filled' : 'ghost'}
            color={band === 'albums' ? 'primary' : 'secondary'}
            icon={mdiImageAlbum}
            title="Albums"
            aria-label="Albums"
            aria-pressed={band === 'albums'}
            disabled={metaBusy}
            onclick={() => (band = band === 'albums' ? null : 'albums')}
          />
        </div>
        <span class="mx-1 h-5 w-px shrink-0 bg-dark/10" aria-hidden="true"></span>
        <DisabledReason locked={!trashable} reason={NOT_OWNER_TRASH} class="flex shrink-0">
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiTrashCanOutline}
            title={hint(`Move ${photos} to the Immich trash`, 'trash')}
            aria-label="Move to Immich trash"
            disabled={metaBusy || !trashable}
            onclick={() => void trash.request(picked)}
          />
        </DisabledReason>
      </ControlBarContent>

      <ControlBarOverflow class="ps-2">
        <IconButton
          size="medium"
          variant="ghost"
          color="secondary"
          icon={mdiClose}
          title={hint('Clear selection', 'gridClearSelection')}
          aria-label="Clear selection"
          disabled={selectingAll}
          onclick={onClear}
        />
      </ControlBarOverflow>
    </ControlBar>

    {#if band === 'tags' && allOwned}
      <BulkTagBand ids={selectedIds} bind:busy />
    {:else if band === 'albums'}
      <BulkAlbumBand ids={selectedIds} bind:busy />
    {/if}
  </div>
{/if}

{#if bulkActionsOpen}
  <BulkActionsDialog onClose={() => (bulkActionsOpen = false)} />
{/if}
