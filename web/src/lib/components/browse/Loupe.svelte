<script lang="ts">
  import { goto } from '$app/navigation';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseView } from '$lib/stores/browseView.svelte';
  import { compare, CENTERED } from '$lib/stores/compare.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { rateAsset, toggleFavorite, toggleReject } from '$lib/cull';
  import { persistedPreviewUrl } from '$lib/api/preview';
  import { getAsset } from '$lib/api/assets';
  import { addTagToAsset, removeTagFromAsset, upsertTags } from '$lib/api/tags';
  import { toasts } from '$lib/stores/toasts.svelte';
  import { metadataConsent } from '$lib/stores/metadataConsent.svelte';
  import { isRejected } from '$lib/reject';
  import { putBounded } from '$lib/utils/boundedRecord';
  import type { TagRef } from '$lib/types/asset';
  import Filmstrip from '$lib/components/shell/Filmstrip.svelte';
  import LoupePane from '$lib/components/browse/LoupePane.svelte';
  import TagPicker from '$lib/components/TagPicker.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import ToolbarButton from '$lib/components/ToolbarButton.svelte';
  import StarRating from '$lib/components/StarRating.svelte';
  import FavoriteButton from '$lib/components/FavoriteButton.svelte';
  import RejectButton from '$lib/components/RejectButton.svelte';
  import { nextRatingFromKey } from '$lib/ratingShortcuts';
  import {
    mdiClose,
    mdiInformationOutline,
    mdiSkipNextOutline,
    mdiKeyboardOutline,
    mdiPencilOutline,
    mdiChevronLeft,
    mdiChevronRight
  } from '@mdi/js';

  const MAX_EDGE = 2560;

  let paneMaxEdge = $state(MAX_EDGE);

  const currentId = $derived(browseView.loupeId);
  const asset = $derived(
    currentId ? (browsing.assets.find((a) => a.id === currentId) ?? null) : null
  );
  const rating = $derived(asset?.exifInfo?.rating ?? 0);
  const rejected = $derived(asset ? isRejected(asset) : false);
  const exif = $derived(asset?.exifInfo ?? null);
  const zoomed = $derived(currentId ? compare.viewOf(currentId).zoomed : false);

  let tagCache = $state<Record<string, TagRef[]>>({});
  let tagOrder = $state<string[]>([]);
  const currentTags = $derived(currentId ? (tagCache[currentId] ?? []) : []);

  $effect(() => {
    const id = currentId;
    if (!id || tagCache[id]) return;
    void getAsset(id)
      .then((a) => setTags(id, a.tags))
      .catch((e: unknown) => toasts.push('error', `tags: ${(e as Error).message}`));
  });

  function setTags(id: string, tags: TagRef[]): void {
    const next = putBounded(tagCache, tagOrder, id, tags, 50);
    tagCache = next.record;
    tagOrder = next.order;
  }

  async function addTag(tag: TagRef): Promise<void> {
    const id = currentId;
    if (!id) return;
    const prev = tagCache[id] ?? [];
    if (prev.some((t) => t.id === tag.id)) return;
    if (!(await metadataConsent.gate())) return;
    setTags(id, [...prev, tag]);
    try {
      await addTagToAsset(tag.id, id);
    } catch (e) {
      setTags(id, prev);
      toasts.push('error', `tag: ${(e as Error).message}`);
    }
  }

  async function removeTag(tagId: string): Promise<void> {
    const id = currentId;
    if (!id) return;
    const prev = tagCache[id] ?? [];
    if (!(await metadataConsent.gate())) return;
    setTags(
      id,
      prev.filter((t) => t.id !== tagId)
    );
    try {
      await removeTagFromAsset(tagId, id);
    } catch (e) {
      setTags(id, prev);
      toasts.push('error', `tag: ${(e as Error).message}`);
    }
  }

  async function createAndAddTag(value: string): Promise<TagRef | null> {
    const id = currentId;
    if (!id) return null;
    if (!(await metadataConsent.gate())) return null;
    try {
      const created = await upsertTags([value]);
      const tag = created[0];
      if (!tag) return null;
      const ref: TagRef = {
        id: tag.id,
        name: tag.name,
        value: tag.value,
        parentId: tag.parentId,
        color: tag.color
      };
      const prev = tagCache[id] ?? [];
      if (!prev.some((t) => t.id === ref.id)) {
        setTags(id, [...prev, ref]);
        try {
          await addTagToAsset(ref.id, id);
        } catch (e) {
          setTags(id, prev);
          toasts.push('error', `tag: ${(e as Error).message}`);
          return null;
        }
      }
      return ref;
    } catch (e) {
      toasts.push('error', `tag: ${(e as Error).message}`);
      return null;
    }
  }

  $effect(() => {
    if (!currentId) return;
    for (const neighbor of [browsing.nextOf(currentId), browsing.prevOf(currentId)]) {
      if (neighbor) {
        const img = new Image();
        img.src = persistedPreviewUrl(neighbor.id, paneMaxEdge);
      }
    }
  });

  function go(delta: number): void {
    if (!currentId) return;
    const next = delta > 0 ? browsing.nextOf(currentId) : browsing.prevOf(currentId);
    if (next) browseView.openLoupe(next.id);
  }

  function toggleZoom(): void {
    if (!currentId) return;
    if (zoomed) compare.applyView(currentId, CENTERED);
    else compare.applyView(currentId, { zoomed: true, cx: 0.5, cy: 0.5 });
  }

  function rate(value: number | null): void {
    if (!currentId) return;
    const id = currentId;
    void rateAsset(id, value).then((ok) => {
      if (ok && browseView.loupeAutoAdvance) go(1);
    });
  }

  function favorite(): void {
    if (!currentId) return;
    void toggleFavorite(currentId).then((ok) => {
      if (ok && browseView.loupeAutoAdvance) go(1);
    });
  }

  function reject(): void {
    if (!currentId) return;
    const id = currentId;
    void toggleReject(id).then((ok) => {
      if (!ok) return;
      const updated = browsing.assets.find((a) => a.id === id);
      if (updated) setTags(id, updated.tags);
      if (browseView.loupeAutoAdvance) go(1);
    });
  }

  function openEditor(): void {
    if (!currentId) return;
    const id = currentId;
    browseView.closeLoupe();
    void goto(`/assets/${id}`);
  }

  function onKeydown(e: KeyboardEvent): void {
    if (!currentId) return;
    const el = document.activeElement;
    if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT')) {
      return;
    }
    if (e.metaKey || e.ctrlKey || e.altKey) return;

    if (e.key === '?' || (e.key === '/' && e.shiftKey)) {
      e.preventDefault();
      ui.toggleKeybindsHelp();
      return;
    }

    switch (e.key) {
      case 'Escape':
        e.preventDefault();
        return browseView.closeLoupe();
      case 'ArrowRight':
      case 'k':
      case 'K':
        e.preventDefault();
        return go(1);
      case 'ArrowLeft':
      case 'j':
      case 'J':
        e.preventDefault();
        return go(-1);
      case 'f':
      case 'F':
        e.preventDefault();
        return favorite();
      case 'x':
      case 'X':
        e.preventDefault();
        return reject();
      case 'z':
      case 'Z':
      case ' ':
        e.preventDefault();
        toggleZoom();
        return;
      case 'i':
      case 'I':
        e.preventDefault();
        browseView.loupeInfoOpen = !browseView.loupeInfoOpen;
        return;
      case 't':
      case 'T':
        e.preventDefault();
        browseView.loupeTagsOpen = !browseView.loupeTagsOpen;
        return;
      case 'e':
      case 'E':
      case 'Enter':
        e.preventDefault();
        return openEditor();
    }
    const next = nextRatingFromKey(e.key, rating);
    if (next !== undefined) {
      e.preventDefault();
      rate(next);
    }
  }

  const hasPrev = $derived(currentId ? browsing.prevOf(currentId) !== null : false);
  const hasNext = $derived(currentId ? browsing.nextOf(currentId) !== null : false);
</script>

<svelte:window onkeydown={onKeydown} />

{#if asset}
  <div class="fixed inset-0 z-40 flex flex-col bg-black/95">
    <div class="flex items-center gap-2 px-4 h-11 flex-none text-immich-dark-fg">
      <span class="text-sm font-medium truncate">{asset.originalFileName}</span>
      <div class="ml-1">
        <StarRating {rating} size={16} onchange={rate} />
      </div>
      <FavoriteButton isFavorite={asset.isFavorite} size={16} ontoggle={() => favorite()} />
      <RejectButton isRejected={rejected} size={16} ontoggle={() => reject()} />

      <div class="ml-2 min-w-0 max-w-[45%]">
        <TagPicker
          tags={currentTags}
          open={browseView.loupeTagsOpen}
          onToggle={() => (browseView.loupeTagsOpen = !browseView.loupeTagsOpen)}
          onClose={() => (browseView.loupeTagsOpen = false)}
          onAdd={addTag}
          onRemove={removeTag}
          onCreate={createAndAddTag}
          anchor="bottom"
        />
      </div>

      <div class="flex-1"></div>

      <ToolbarButton
        path={mdiInformationOutline}
        size={18}
        title="Info (I)"
        active={browseView.loupeInfoOpen}
        onclick={() => (browseView.loupeInfoOpen = !browseView.loupeInfoOpen)}
      />
      <ToolbarButton
        path={mdiSkipNextOutline}
        size={18}
        title="Auto-advance after rating"
        active={browseView.loupeAutoAdvance}
        pressed={browseView.loupeAutoAdvance}
        onclick={() => browseView.setLoupeAutoAdvance(!browseView.loupeAutoAdvance)}
      />
      <ToolbarButton
        path={mdiKeyboardOutline}
        size={18}
        title="Keyboard shortcuts (?)"
        onclick={() => ui.toggleKeybindsHelp()}
      />
      <ToolbarButton
        path={mdiPencilOutline}
        size={18}
        title="Edit (E)"
        onclick={openEditor}
      />
      <ToolbarButton
        path={mdiClose}
        size={18}
        title="Close (Esc)"
        onclick={() => browseView.closeLoupe()}
      />
    </div>

    <div class="flex-1 min-h-0 relative flex">
      {#if currentId && asset}
        <LoupePane
          assetId={currentId}
          alt={asset.originalFileName}
          view={compare.viewOf(currentId)}
          onView={(next) => compare.applyView(currentId, next)}
          onSize={(size) => (paneMaxEdge = size)}
        />
      {/if}

      {#if hasPrev}
        <button
          type="button"
          class="absolute left-2 top-1/2 -translate-y-1/2 p-2 rounded-full bg-black/40 hover:bg-black/70 text-white"
          title="Previous (←)"
          onclick={() => go(-1)}
        >
          <Icon path={mdiChevronLeft} size={24} />
        </button>
      {/if}
      {#if hasNext}
        <button
          type="button"
          class="absolute right-2 top-1/2 -translate-y-1/2 p-2 rounded-full bg-black/40 hover:bg-black/70 text-white"
          title="Next (→)"
          onclick={() => go(1)}
        >
          <Icon path={mdiChevronRight} size={24} />
        </button>
      {/if}

      {#if browseView.loupeInfoOpen && exif}
        <div
          class="absolute top-2 right-2 w-56 bg-immich-dark-gray/95 border border-white/10 rounded-lg p-3 text-[11px] text-immich-dark-fg/80 flex flex-col gap-1"
        >
          {#if exif.make || exif.model}
            <span>{[exif.make, exif.model].filter(Boolean).join(' ')}</span>
          {/if}
          {#if exif.lensModel}<span class="text-immich-dark-fg/50">{exif.lensModel}</span>{/if}
          <div class="flex flex-wrap gap-x-3 gap-y-0.5 text-immich-dark-fg/60">
            {#if exif.fNumber}<span>ƒ/{exif.fNumber}</span>{/if}
            {#if exif.exposureTime}<span>{exif.exposureTime}s</span>{/if}
            {#if exif.iso}<span>ISO {exif.iso}</span>{/if}
            {#if exif.focalLength}<span>{exif.focalLength}mm</span>{/if}
          </div>
          {#if exif.exifImageWidth && exif.exifImageHeight}
            <span class="text-immich-dark-fg/50">{exif.exifImageWidth} × {exif.exifImageHeight}</span>
          {/if}
          {#if exif.dateTimeOriginal}
            <span class="text-immich-dark-fg/50">{exif.dateTimeOriginal}</span>
          {/if}
        </div>
      {/if}
    </div>

    <Filmstrip currentId={currentId} onSelect={(id) => browseView.openLoupe(id)} size={72} showBadges />
  </div>
{/if}
