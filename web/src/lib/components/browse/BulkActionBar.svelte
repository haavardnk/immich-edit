<script lang="ts">
  import type { AssetSummary } from '$lib/types/album';
  import type { AssetDetail } from '$lib/types/asset';
  import { selection } from '$lib/stores/selection.svelte';
  import { hint } from '$lib/keybinds';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { addTagToAsset, removeTagFromAsset } from '$lib/api/tags';
  import { updateAsset } from '$lib/api/assets';
  import { createVirtualCopy } from '$lib/copies';
  import { metadataConsent } from '$lib/stores/metadataConsent.svelte';
  import { rejected } from '$lib/stores/rejected.svelte';
  import { ensureRejectTag, setRejectedTags } from '$lib/reject';
  import { toasts } from '$lib/stores/toasts.svelte';
  import BulkActionsDialog from './BulkActionsDialog.svelte';
  import BulkTagBand from './BulkTagBand.svelte';
  import {
    Button,
    ControlBar,
    ControlBarContent,
    ControlBarHeader,
    ControlBarOverflow,
    IconButton
  } from '@immich/ui';
  import type { MultiMode } from '$lib/compareEntry';
  import {
    mdiClose,
    mdiCloseCircle,
    mdiCloseCircleOutline,
    mdiCompare,
    mdiContentDuplicate,
    mdiHeart,
    mdiHeartOutline,
    mdiStar,
    mdiStarOutline,
    mdiSelectAll,
    mdiTagOutline,
    mdiTuneVariant,
    mdiViewGridOutline
  } from '@mdi/js';

  let { assets, onMulti }: { assets: AssetSummary[]; onMulti: (mode: MultiMode) => void } =
    $props();

  let busy = $state(false);
  let showTags = $state(false);
  let bulkActionsOpen = $state(false);

  let metaBusy = $derived(busy || selection.allFiltered);
  let canCompare = $derived(!selection.allFiltered && selection.count >= 2);
  let showSelectAll = $derived(
    browsing.query !== null && browsing.total !== undefined && browsing.total > 0
  );

  function loadedActionLabel(action: string): string {
    return selection.allFiltered ? `${action} unavailable: select loaded assets` : action;
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
    if (busy || selection.allFiltered) return;
    if (!(await metadataConsent.gate())) return;
    busy = true;
    const ids = [...selection.selected];
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
    if (busy || selection.allFiltered) return;
    busy = true;
    const ids = [...selection.selected];
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

  function setRating(value: number): void {
    void applyMeta((id) => updateAsset(id, { rating: value }));
  }

  async function applyReject(value: boolean): Promise<void> {
    if (busy || selection.allFiltered) return;
    if (!(await metadataConsent.gate())) return;
    const rejectTag = await ensureRejectTag();
    if (!rejectTag) {
      toasts.push('error', 'reject: could not create tag', 6000);
      return;
    }
    busy = true;
    const ids = [...selection.selected];
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
</script>

{#if selection.active}
  <div
    class="fixed bottom-4 left-1/2 z-40 flex w-max max-w-[calc(100vw-2rem)] -translate-x-1/2 flex-col overflow-hidden rounded-lg bg-light-100 shadow-2xl"
  >
    <ControlBar static shape="rectangle" class="min-w-0 px-3">
      <ControlBarHeader class="pe-2">
        <span class="whitespace-nowrap text-sm font-medium" aria-live="polite"
          >{selection.targetCount} selected</span
        >
      </ControlBarHeader>

      <ControlBarContent class="min-w-0 gap-1 overflow-x-auto overscroll-contain scrollbar-hidden">
        <div class="flex shrink-0 items-center gap-1">
          <Button
            size="small"
            variant="ghost"
            color="secondary"
            leadingIcon={mdiSelectAll}
            onclick={() => selection.selectLoaded(assets.map((a) => a.id))}
          >
            Select loaded
          </Button>
          {#if showSelectAll}
            <Button
              size="small"
              variant="ghost"
              color="secondary"
              leadingIcon={mdiSelectAll}
              title="Select all (job-based actions only)"
              onclick={() => selection.selectFiltered(browsing.query!, browsing.total!)}
            >
              Select all
            </Button>
          {/if}
        </div>

        <div class="ms-2 flex shrink-0 items-center gap-1">
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiCompare}
            title={loadedActionLabel(hint('Compare selected', 'enterCompare'))}
            aria-label={loadedActionLabel('Compare selected')}
            disabled={!canCompare}
            onclick={() => onMulti('compare')}
          />
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiViewGridOutline}
            title={loadedActionLabel(hint('Survey selected', 'enterSurvey'))}
            aria-label={loadedActionLabel('Survey selected')}
            disabled={!canCompare}
            onclick={() => onMulti('survey')}
          />
        </div>

        <div class="ms-2 flex shrink-0 items-center gap-1">
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiHeart}
            title={loadedActionLabel('Favorite')}
            aria-label={loadedActionLabel('Favorite')}
            disabled={metaBusy}
            onclick={() => setFavorite(true)}
          />
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiHeartOutline}
            title={loadedActionLabel('Unfavorite')}
            aria-label={loadedActionLabel('Unfavorite')}
            disabled={metaBusy}
            onclick={() => setFavorite(false)}
          />
        </div>

        <div class="ms-2 flex shrink-0 items-center gap-0.5" role="group" aria-label="Set rating">
          {#each [1, 2, 3, 4, 5] as n (n)}
            <IconButton
              size="medium"
              variant="ghost"
              color="secondary"
              icon={mdiStar}
              title={loadedActionLabel(`Rate ${n}`)}
              aria-label={loadedActionLabel(`Rate ${n}`)}
              disabled={metaBusy}
              onclick={() => setRating(n)}
            />
          {/each}
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiStarOutline}
            title={loadedActionLabel('Clear rating')}
            aria-label={loadedActionLabel('Clear rating')}
            disabled={metaBusy}
            onclick={() => setRating(0)}
          />
        </div>

        <div class="ms-2 flex shrink-0 items-center gap-1">
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiCloseCircle}
            title={loadedActionLabel('Reject')}
            aria-label={loadedActionLabel('Reject')}
            disabled={metaBusy}
            onclick={() => void applyReject(true)}
          />
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiCloseCircleOutline}
            title={loadedActionLabel('Unreject')}
            aria-label={loadedActionLabel('Unreject')}
            disabled={metaBusy}
            onclick={() => void applyReject(false)}
          />
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiContentDuplicate}
            title={loadedActionLabel(hint('Create a virtual copy', 'createVirtualCopy'))}
            aria-label={loadedActionLabel('Create virtual copy')}
            disabled={metaBusy}
            onclick={() => void createCopies()}
          />
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiTuneVariant}
            title="Edit and export selected"
            aria-label="Edit and export selected"
            onclick={() => (bulkActionsOpen = true)}
          />
          <IconButton
            size="medium"
            variant={showTags ? 'filled' : 'ghost'}
            color={showTags ? 'primary' : 'secondary'}
            icon={mdiTagOutline}
            title={loadedActionLabel('Tags')}
            aria-label={loadedActionLabel('Tags')}
            aria-pressed={showTags}
            disabled={metaBusy}
            onclick={() => (showTags = !showTags)}
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
          onclick={selection.clear}
        />
      </ControlBarOverflow>
    </ControlBar>

    {#if selection.allFiltered}
      <div
        role="status"
        class="border-t border-hairline px-3 py-1 text-center text-[10px] text-dark/65"
      >
        Job actions only. Select loaded assets to use metadata, compare, tags, or copies.
      </div>
    {/if}

    {#if showTags}
      <BulkTagBand {runPool} bind:busy />
    {/if}
  </div>
{/if}

{#if bulkActionsOpen}
  <BulkActionsDialog onClose={() => (bulkActionsOpen = false)} />
{/if}
