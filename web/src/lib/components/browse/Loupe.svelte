<script lang="ts">
  import { goto } from '$app/navigation';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseView } from '$lib/stores/browseView.svelte';
  import { rateAsset, toggleFavorite } from '$lib/cull';
  import { persistedPreviewUrl } from '$lib/api/preview';
  import { getAsset } from '$lib/api/assets';
  import { addTagToAsset, removeTagFromAsset, upsertTags } from '$lib/api/tags';
  import { toasts } from '$lib/stores/toasts.svelte';
  import type { TagRef } from '$lib/types/asset';
  import Filmstrip from '$lib/components/shell/Filmstrip.svelte';
  import TagPicker from '$lib/components/TagPicker.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import {
    mdiClose,
    mdiHeart,
    mdiHeartOutline,
    mdiStar,
    mdiStarOutline,
    mdiInformationOutline,
    mdiSkipNextOutline,
    mdiPencilOutline,
    mdiChevronLeft,
    mdiChevronRight
  } from '@mdi/js';

  const MAX_EDGE = 2560;

  const currentId = $derived(browseView.loupeId);
  const asset = $derived(
    currentId ? (browsing.assets.find((a) => a.id === currentId) ?? null) : null
  );
  const rating = $derived(asset?.exifInfo?.rating ?? 0);
  const exif = $derived(asset?.exifInfo ?? null);
  const previewSrc = $derived(currentId ? persistedPreviewUrl(currentId, MAX_EDGE) : '');

  let tagCache = $state<Record<string, TagRef[]>>({});
  const currentTags = $derived(currentId ? (tagCache[currentId] ?? []) : []);

  $effect(() => {
    const id = currentId;
    if (!id || tagCache[id]) return;
    void getAsset(id)
      .then((a) => (tagCache = { ...tagCache, [id]: a.tags }))
      .catch((e: unknown) => toasts.push('error', `tags: ${(e as Error).message}`));
  });

  function setTags(id: string, tags: TagRef[]): void {
    tagCache = { ...tagCache, [id]: tags };
  }

  async function addTag(tag: TagRef): Promise<void> {
    const id = currentId;
    if (!id) return;
    const prev = tagCache[id] ?? [];
    if (prev.some((t) => t.id === tag.id)) return;
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
        img.src = persistedPreviewUrl(neighbor.id, MAX_EDGE);
      }
    }
  });

  function go(delta: number): void {
    if (!currentId) return;
    const next = delta > 0 ? browsing.nextOf(currentId) : browsing.prevOf(currentId);
    if (next) browseView.openLoupe(next.id);
  }

  function rate(value: number | null): void {
    if (!currentId) return;
    void rateAsset(currentId, value);
    if (browseView.loupeAutoAdvance) go(1);
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
        return void toggleFavorite(currentId);
      case 'z':
      case 'Z':
      case ' ':
        e.preventDefault();
        browseView.loupeZoomed = !browseView.loupeZoomed;
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
    if (e.key >= '0' && e.key <= '5') {
      e.preventDefault();
      rate(e.key === '0' ? null : Number(e.key));
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
      <div class="flex items-center gap-0.5 ml-1">
        {#each [1, 2, 3, 4, 5] as n (n)}
          <button
            type="button"
            class="text-immich-dark-fg/70 hover:text-immich-dark-fg"
            title={`Rate ${n}`}
            onclick={() => rate(n === rating ? null : n)}
          >
            <Icon path={n <= rating ? mdiStar : mdiStarOutline} size={16} />
          </button>
        {/each}
      </div>
      <button
        type="button"
        class="ml-1 {asset.isFavorite ? 'text-red-400' : 'text-immich-dark-fg/70 hover:text-immich-dark-fg'}"
        title="Favorite (F)"
        onclick={() => toggleFavorite(asset.id)}
      >
        <Icon path={asset.isFavorite ? mdiHeart : mdiHeartOutline} size={16} />
      </button>

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

      <button
        type="button"
        class="p-1 rounded hover:bg-white/10 {browseView.loupeInfoOpen
          ? 'text-immich-dark-primary'
          : 'text-immich-dark-fg/70'}"
        title="Info (I)"
        onclick={() => (browseView.loupeInfoOpen = !browseView.loupeInfoOpen)}
      >
        <Icon path={mdiInformationOutline} size={18} />
      </button>
      <button
        type="button"
        class="p-1 rounded hover:bg-white/10 {browseView.loupeAutoAdvance
          ? 'text-immich-dark-primary'
          : 'text-immich-dark-fg/70'}"
        title="Auto-advance after rating"
        aria-pressed={browseView.loupeAutoAdvance}
        onclick={() => browseView.setLoupeAutoAdvance(!browseView.loupeAutoAdvance)}
      >
        <Icon path={mdiSkipNextOutline} size={18} />
      </button>
      <button
        type="button"
        class="p-1 rounded hover:bg-white/10 text-immich-dark-fg/70"
        title="Edit (E)"
        onclick={openEditor}
      >
        <Icon path={mdiPencilOutline} size={18} />
      </button>
      <button
        type="button"
        class="p-1 rounded hover:bg-white/10 text-immich-dark-fg/70"
        title="Close (Esc)"
        onclick={() => browseView.closeLoupe()}
      >
        <Icon path={mdiClose} size={18} />
      </button>
    </div>

    <div class="flex-1 min-h-0 relative flex">
      <button
        type="button"
        class="flex-1 min-w-0 flex items-center justify-center {browseView.loupeZoomed
          ? 'overflow-auto cursor-zoom-out'
          : 'overflow-hidden cursor-zoom-in'}"
        aria-label={browseView.loupeZoomed ? 'Zoom out' : 'Zoom in'}
        onclick={() => (browseView.loupeZoomed = !browseView.loupeZoomed)}
      >
        <img
          src={previewSrc}
          alt={asset.originalFileName}
          class={browseView.loupeZoomed ? 'max-w-none' : 'max-w-full max-h-full object-contain'}
        />
      </button>

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
