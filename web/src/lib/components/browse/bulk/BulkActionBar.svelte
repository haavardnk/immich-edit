<script lang="ts">
  import type { AssetSummary } from '$lib/types/album';
  import type { AssetDetail } from '$lib/types/asset';
  import { page } from '$app/state';
  import { hint } from '$lib/keybinds';
  import { editorHref } from '$lib/editorNavigation';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { addTagToAsset, removeTagFromAsset } from '$lib/api/tags';
  import { updateAsset } from '$lib/api/assets';
  import { createVirtualCopy } from '$lib/copies';
  import { metadataConsent } from '$lib/stores/metadataConsent.svelte';
  import { rejected } from '$lib/stores/rejected.svelte';
  import { ensureRejectTag, isRejected, setRejectedTags } from '$lib/reject';
  import {
    LABEL_NAMES,
    labelOf,
    labelTagFor,
    withLabel,
    writeLabel,
    type LabelColor
  } from '$lib/labels';
  import { assignLabel } from '$lib/stores/labels.svelte';
  import LabelPicker from '$lib/components/LabelPicker.svelte';
  import FavoriteButton from '$lib/components/FavoriteButton.svelte';
  import RejectButton from '$lib/components/RejectButton.svelte';
  import StarRating from '$lib/components/StarRating.svelte';
  import { commonValue, coverage } from '$lib/utils/selectionState';
  import { toasts } from '$lib/stores/toasts.svelte';
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
  import type { MultiMode } from '$lib/compareEntry';
  import { MAX_PANES } from '$lib/stores/compare.svelte';
  import {
    mdiClose,
    mdiImageAlbum,
    mdiCompare,
    mdiContentDuplicate,
    mdiImageEditOutline,
    mdiTagOutline,
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

  let metaBusy = $derived(busy || selectingAll);
  let count = $derived(selectedIds.length);
  let targetCount = $derived(selectingAll ? assets.length : count);
  let picked = $derived.by(() => {
    const ids = new Set(selectedIds);
    return assets.filter((a) => ids.has(a.id));
  });
  let single = $derived(!selectingAll && count === 1 ? (picked[0] ?? null) : null);
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

  async function runPool<T>(
    items: T[],
    limit: number,
    fn: (item: T) => Promise<void>
  ): Promise<void> {
    let next = 0;
    const workers = Array.from({ length: Math.min(limit, items.length) }, async () => {
      while (next < items.length) {
        const item = items[next++];
        if (item !== undefined) await fn(item);
      }
    });
    await Promise.all(workers);
  }

  async function applyMeta(fn: (id: string) => Promise<AssetDetail>): Promise<void> {
    if (busy || selectingAll) return;
    if (!(await metadataConsent.gate())) return;
    busy = true;
    const ids = [...selectedIds];
    const byId = new Map(assets.map((a) => [a.id, a]));
    let failed = 0;
    await runPool(ids, 6, async (id) => {
      try {
        const updated = await fn(id);
        const a = byId.get(id);
        if (a) {
          a.isFavorite = updated.isFavorite;
          a.exifInfo = updated.exifInfo;
        }
        browsing.patch(id, { isFavorite: updated.isFavorite, exifInfo: updated.exifInfo });
      } catch {
        failed += 1;
      }
    });
    busy = false;
    if (failed > 0) {
      toasts.push('warn', `${failed} of ${ids.length} failed`, 6000);
    } else {
      toasts.push('success', `Updated ${ids.length} asset${ids.length === 1 ? '' : 's'}`, 4000);
    }
  }

  function setFavorite(value: boolean): void {
    void applyMeta((id) => updateAsset(id, { isFavorite: value }));
  }

  async function createCopies(): Promise<void> {
    if (busy || selectingAll) return;
    busy = true;
    const ids = [...selectedIds];
    let failed = 0;
    await runPool(ids, 4, async (id) => {
      try {
        await createVirtualCopy(id, { navigate: false });
      } catch {
        failed += 1;
      }
    });
    busy = false;
    if (failed > 0) {
      toasts.push('warn', `${failed} of ${ids.length} failed`, 6000);
    } else {
      toasts.push(
        'success',
        `Created ${ids.length} virtual cop${ids.length === 1 ? 'y' : 'ies'}`,
        4000
      );
    }
  }

  function setRating(value: number | null): void {
    void applyMeta((id) => updateAsset(id, { rating: value }));
  }

  async function applyReject(value: boolean): Promise<void> {
    if (busy || selectingAll) return;
    if (!(await metadataConsent.gate())) return;
    const rejectTag = await ensureRejectTag();
    if (!rejectTag) {
      toasts.push('error', 'reject: could not create tag', 6000);
      return;
    }
    busy = true;
    const ids = [...selectedIds];
    const byId = new Map(assets.map((a) => [a.id, a]));
    let failed = 0;
    await runPool(ids, 6, async (id) => {
      try {
        await (value ? addTagToAsset(rejectTag.id, id) : removeTagFromAsset(rejectTag.id, id));
        const a = byId.get(id);
        if (a) {
          const tags = setRejectedTags(a.tags, rejectTag, value);
          a.tags = tags;
          browsing.patch(id, { tags });
        }
        if (value) rejected.add(id, rejectTag);
        else rejected.remove(id);
      } catch {
        failed += 1;
      }
    });
    busy = false;
    if (failed > 0) {
      toasts.push('warn', `${failed} of ${ids.length} failed`, 6000);
    } else {
      toasts.push(
        'success',
        `${value ? 'Rejected' : 'Unrejected'} ${ids.length} asset${ids.length === 1 ? '' : 's'}`,
        4000
      );
    }
  }
  async function applyLabel(color: LabelColor | null): Promise<void> {
    if (busy || selectingAll) return;
    if (!(await metadataConsent.gate())) return;
    const tag = await labelTagFor(color);
    if (tag === undefined) {
      toasts.push('error', 'label: could not create tag', 6000);
      return;
    }
    busy = true;
    const ids = [...selectedIds];
    const byId = new Map(assets.map((a) => [a.id, a]));
    let failed = 0;
    await runPool(ids, 6, async (id) => {
      const a = byId.get(id);
      try {
        await writeLabel(id, a?.tags ?? [], tag);
        if (a) {
          const tags = withLabel(a.tags, tag);
          a.tags = tags;
          browsing.patch(id, { tags });
        }
        assignLabel(id, color, tag);
      } catch {
        failed += 1;
      }
    });
    busy = false;
    if (failed > 0) {
      toasts.push('warn', `${failed} of ${ids.length} failed`, 6000);
    } else {
      const what = color ? `${LABEL_NAMES[color]} label on` : 'Cleared the label of';
      toasts.push('success', `${what} ${ids.length} photo${ids.length === 1 ? '' : 's'}`, 4000);
    }
  }
</script>

{#if count > 0 || selectingAll}
  <div
    bind:this={bar}
    role="toolbar"
    aria-label="Selection actions"
    class="fixed bottom-4 left-1/2 z-40 flex w-max max-w-[calc(100vw-2rem)] -translate-x-1/2 flex-col overflow-hidden rounded-lg bg-light-100 shadow-2xl"
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
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiImageEditOutline}
            title={hint('Open in editor', 'openEditor')}
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

        <div class="flex shrink-0 items-center gap-1">
          <FavoriteButton
            size="medium"
            isFavorite={favoriteState === 'all'}
            mixed={favoriteState === 'some'}
            disabled={metaBusy}
            ontoggle={() => setFavorite(favoriteState !== 'all')}
          />
          <StarRating
            size={20}
            rating={commonRating ?? 0}
            mixed={commonRating === null}
            disabled={metaBusy}
            onchange={setRating}
          />
          <RejectButton
            size="medium"
            isRejected={rejectState === 'all'}
            mixed={rejectState === 'some'}
            disabled={metaBusy}
            ontoggle={() => void applyReject(rejectState !== 'all')}
          />
          <LabelPicker
            size="medium"
            label={commonLabel}
            disabled={metaBusy}
            onchange={(color) => void applyLabel(color)}
          />
        </div>
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
          <IconButton
            size="medium"
            variant={band === 'tags' ? 'filled' : 'ghost'}
            color={band === 'tags' ? 'primary' : 'secondary'}
            icon={mdiTagOutline}
            title="Tags"
            aria-label="Tags"
            aria-pressed={band === 'tags'}
            disabled={metaBusy}
            onclick={() => (band = band === 'tags' ? null : 'tags')}
          />
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

    {#if band === 'tags'}
      <BulkTagBand ids={selectedIds} {runPool} bind:busy />
    {:else if band === 'albums'}
      <BulkAlbumBand ids={selectedIds} bind:busy />
    {/if}
  </div>
{/if}

{#if bulkActionsOpen}
  <BulkActionsDialog onClose={() => (bulkActionsOpen = false)} />
{/if}
