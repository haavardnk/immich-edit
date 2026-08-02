<script lang="ts">
  import type { AssetSummary } from '$lib/types/album';
  import type { AssetDetail } from '$lib/types/asset';
  import { selection } from '$lib/stores/selection.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { listTags, addTagToAsset, removeTagFromAsset, type TagSummary } from '$lib/api/tags';
  import { updateAsset } from '$lib/api/assets';
  import { metadataConsent } from '$lib/stores/metadataConsent.svelte';
  import { rejected } from '$lib/stores/rejected.svelte';
  import { ensureRejectTag, isManagedTag, setRejectedTags, toTagRef } from '$lib/reject';
  import { toasts } from '$lib/stores/toasts.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import MultiSelect from '$lib/components/MultiSelect.svelte';
  import {
    mdiClose,
    mdiCloseCircle,
    mdiCloseCircleOutline,
    mdiHeart,
    mdiHeartOutline,
    mdiStar,
    mdiStarOutline,
    mdiSelectAll,
    mdiTagOutline,
  } from '@mdi/js';

  let { assets }: { assets: AssetSummary[] } = $props();

  let busy = $state(false);
  let tags = $state<TagSummary[]>([]);
  let tagsLoaded = $state(false);
  let showTags = $state(false);
  let chosenTags = $state<string[]>([]);

  let metaBusy = $derived(busy || selection.allFiltered);
  let showSelectAll = $derived(
    browsing.query !== null && browsing.total !== undefined && browsing.total > 0,
  );

  $effect(() => {
    if (selection.active && !tagsLoaded) {
      tagsLoaded = true;
      listTags()
        .then((t) => {
          tags = t.filter((tag) => !isManagedTag(toTagRef(tag)));
        })
        .catch(() => (tagsLoaded = false));
    }
  });

  async function runPool<T>(items: T[], limit: number, fn: (item: T) => Promise<void>): Promise<void> {
    let next = 0;
    const workers = Array.from({ length: Math.min(limit, items.length) }, async () => {
      while (next < items.length) {
        const idx = next++;
        await fn(items[idx]);
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
      toasts.push('success', `Updated ${ids.length} assets`, 4000);
    }
  }

  function setFavorite(value: boolean): void {
    void applyMeta((id) => updateAsset(id, { isFavorite: value }));
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
      toasts.push('success', `${value ? 'Rejected' : 'Unrejected'} ${ids.length} assets`, 4000);
    }
  }

  async function applyTags(add: boolean): Promise<void> {
    if (busy || selection.allFiltered || chosenTags.length === 0) return;
    if (!(await metadataConsent.gate())) return;
    busy = true;
    const ids = [...selection.selected];
    let failed = 0;
    const pairs = ids.flatMap((id) => chosenTags.map((tagId) => ({ id, tagId })));
    await runPool(pairs, 6, async ({ id, tagId }) => {
      try {
        await (add ? addTagToAsset(tagId, id) : removeTagFromAsset(tagId, id));
      } catch {
        failed += 1;
      }
    });
    busy = false;
    if (failed > 0) {
      toasts.push('warn', `${failed} tag updates failed`, 6000);
    } else {
      toasts.push('success', `${add ? 'Added' : 'Removed'} tags on ${ids.length} assets`, 4000);
    }
    chosenTags = [];
  }
</script>

{#if selection.active}
  <div
    class="fixed bottom-4 left-1/2 -translate-x-1/2 z-40 flex flex-col gap-2 bg-immich-dark-gray border border-white/10 rounded-xl shadow-2xl px-3 py-2 max-w-[95vw]"
  >
    <div class="flex items-center gap-2 flex-wrap text-xs">
      <span class="font-medium px-1">{selection.targetCount} selected</span>
      <button
        class="flex items-center gap-1 px-2 py-1 rounded-lg hover:bg-white/10 transition-colors"
        onclick={() => selection.selectLoaded(assets.map((a) => a.id))}
        title="Select loaded"
      >
        <Icon path={mdiSelectAll} size={16} />
        Select loaded
      </button>
      {#if showSelectAll}
        <button
          class="flex items-center gap-1 px-2 py-1 rounded-lg hover:bg-white/10 transition-colors"
          onclick={() => selection.selectFiltered(browsing.query!, browsing.total!)}
          title="Select all (job-based actions only)"
        >
          <Icon path={mdiSelectAll} size={16} />
          Select all
        </button>
      {/if}

      <div class="w-px h-5 bg-white/10"></div>

      <button
        class="p-1.5 rounded-lg hover:bg-white/10 transition-colors disabled:opacity-40"
        disabled={metaBusy}
        onclick={() => setFavorite(true)}
        title="Favorite"
        aria-label="Favorite"
      >
        <Icon path={mdiHeart} size={16} />
      </button>
      <button
        class="p-1.5 rounded-lg hover:bg-white/10 transition-colors disabled:opacity-40"
        disabled={metaBusy}
        onclick={() => setFavorite(false)}
        title="Unfavorite"
        aria-label="Unfavorite"
      >
        <Icon path={mdiHeartOutline} size={16} />
      </button>

      <div class="w-px h-5 bg-white/10"></div>

      <div class="flex items-center gap-0.5" title="Set rating">
        {#each [1, 2, 3, 4, 5] as n (n)}
          <button
            class="p-0.5 rounded hover:bg-white/10 transition-colors disabled:opacity-40"
            disabled={metaBusy}
            onclick={() => setRating(n)}
            aria-label={`Rate ${n}`}
          >
            <Icon path={mdiStar} size={16} />
          </button>
        {/each}
        <button
          class="p-0.5 rounded hover:bg-white/10 transition-colors disabled:opacity-40"
          disabled={metaBusy}
          onclick={() => setRating(0)}
          aria-label="Clear rating"
        >
          <Icon path={mdiStarOutline} size={16} />
        </button>
      </div>

      <div class="w-px h-5 bg-white/10"></div>

      <button
        class="p-1.5 rounded-lg hover:bg-white/10 transition-colors disabled:opacity-40"
        disabled={metaBusy}
        onclick={() => void applyReject(true)}
        title="Reject"
        aria-label="Reject"
      >
        <Icon path={mdiCloseCircle} size={16} />
      </button>
      <button
        class="p-1.5 rounded-lg hover:bg-white/10 transition-colors disabled:opacity-40"
        disabled={metaBusy}
        onclick={() => void applyReject(false)}
        title="Unreject"
        aria-label="Unreject"
      >
        <Icon path={mdiCloseCircleOutline} size={16} />
      </button>

      <div class="w-px h-5 bg-white/10"></div>

      <button
        class="flex items-center gap-1 px-2 py-1 rounded-lg hover:bg-white/10 transition-colors disabled:opacity-40 {showTags ? 'bg-white/10' : ''}"
        disabled={metaBusy}
        onclick={() => (showTags = !showTags)}
        title="Tags"
      >
        <Icon path={mdiTagOutline} size={16} />
        Tags
      </button>

      <div class="w-px h-5 bg-white/10"></div>

      <button
        class="p-1.5 rounded-lg hover:bg-white/10 transition-colors"
        onclick={selection.clear}
        title="Clear selection (Esc)"
        aria-label="Clear selection"
      >
        <Icon path={mdiClose} size={16} />
      </button>
    </div>

    {#if showTags}
      <div class="flex flex-col gap-1.5 border-t border-white/10 pt-2">
        <span class="text-[11px] text-white/50 px-1">
          Pick tags, then apply them to the {selection.count} selected asset{selection.count === 1
            ? ''
            : 's'}
        </span>
        <div class="flex items-end gap-2">
          <div class="flex-1 min-w-55">
            <MultiSelect
              options={tags}
              bind:selected={chosenTags}
              getId={(t) => t.id}
              getLabel={(t) => t.value}
              placeholder="Choose tags…"
              dropUp
            />
          </div>
          <button
            class="px-2.5 py-1 rounded-lg bg-immich-primary/90 hover:bg-immich-primary text-white text-xs whitespace-nowrap transition-colors disabled:opacity-40 disabled:hover:bg-immich-primary/90"
            disabled={metaBusy || chosenTags.length === 0}
            onclick={() => void applyTags(true)}
          >
            Add to selected
          </button>
          <button
            class="px-2.5 py-1 rounded-lg bg-white/5 hover:bg-white/10 text-xs whitespace-nowrap transition-colors disabled:opacity-40"
            disabled={metaBusy || chosenTags.length === 0}
            onclick={() => void applyTags(false)}
          >
            Remove from selected
          </button>
        </div>
      </div>
    {/if}
  </div>
{/if}
