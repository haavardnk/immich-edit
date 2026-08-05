<script lang="ts">
  import { goto } from '$app/navigation';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseView } from '$lib/stores/browseView.svelte';
  import { compare, CENTERED } from '$lib/stores/compare.svelte';
  import { selection } from '$lib/stores/selection.svelte';
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
    mdiCompare,
    mdiInformationOutline,
    mdiSkipNextOutline,
    mdiKeyboardOutline,
    mdiPencilOutline,
    mdiChevronLeft,
    mdiChevronRight,
    mdiViewGridOutline
  } from '@mdi/js';

  const MAX_EDGE = 2560;

  let paneMaxEdge = $state(MAX_EDGE);

  const currentId = $derived(browseView.loupeId);
  const multi = $derived(compare.mode !== 'single');
  const panes = $derived(multi ? compare.members : currentId ? [currentId] : []);
  const focusedId = $derived(multi ? compare.focusedId : currentId);
  const asset = $derived(
    focusedId ? (browsing.assets.find((a) => a.id === focusedId) ?? null) : null
  );
  const rating = $derived(asset?.exifInfo?.rating ?? 0);
  const rejected = $derived(asset ? isRejected(asset) : false);
  const exif = $derived(asset?.exifInfo ?? null);
  const zoomed = $derived(focusedId ? compare.viewOf(focusedId).zoomed : false);
  const cols = $derived(panes.length <= 4 ? 2 : 3);
  const gridStyle = $derived(
    multi ? `grid-template-columns: repeat(${Math.min(cols, panes.length)}, minmax(0, 1fr));` : ''
  );

  let tagCache = $state<Record<string, TagRef[]>>({});
  let tagOrder = $state<string[]>([]);
  const currentTags = $derived(focusedId ? (tagCache[focusedId] ?? []) : []);

  $effect(() => {
    const id = focusedId;
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
    const id = focusedId;
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
    const id = focusedId;
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
    const id = focusedId;
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
    if (multi || !currentId) return;
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

  function advanceFocused(delta: number): void {
    const id = compare.focusedId;
    if (!id) return;
    let cursor = id;
    for (;;) {
      const next = delta > 0 ? browsing.nextOf(cursor) : browsing.prevOf(cursor);
      if (!next) return;
      if (!compare.members.includes(next.id)) {
        compare.setMember(compare.focusIndex, next.id);
        return;
      }
      cursor = next.id;
    }
  }

  function selectedInOrder(): string[] {
    return browsing.assets.filter((a) => selection.selected.has(a.id)).map((a) => a.id);
  }

  function compareMembers(): string[] {
    const ordered = selectedInOrder();
    if (ordered.length >= 2) return ordered.slice(0, 2);
    if (!currentId) return [];
    const next = browsing.nextOf(currentId);
    return next ? [currentId, next.id] : [currentId];
  }

  function surveyMembers(): string[] {
    const ordered = selectedInOrder();
    if (ordered.length >= 2) return ordered.slice(0, 9);
    if (!currentId) return [];
    const start = browsing.assets.findIndex((a) => a.id === currentId);
    if (start < 0) return [currentId];
    return browsing.assets.slice(start, start + 6).map((a) => a.id);
  }

  function enterCompare(): void {
    if (multi) {
      leaveMulti();
      return;
    }
    const members = compareMembers();
    if (members.length < 2) {
      toasts.push('info', 'compare needs two photos');
      return;
    }
    compare.enter('compare', members);
  }

  function enterSurvey(): void {
    if (multi) {
      leaveMulti();
      return;
    }
    const members = surveyMembers();
    if (members.length < 2) {
      toasts.push('info', 'survey needs two photos');
      return;
    }
    compare.enter('survey', members);
  }

  function leaveMulti(): void {
    const id = compare.focusedId;
    const survivors = compare.mode === 'survey' && compare.pruned ? [...compare.members] : [];
    compare.exit();
    if (survivors.length > 0) selection.selectLoaded(survivors);
    if (id) browseView.openLoupe(id);
  }

  function dropFocused(): void {
    if (compare.members.length <= 1) return;
    compare.drop(compare.focusIndex);
  }

  function toggleZoom(): void {
    const id = focusedId;
    if (!id) return;
    if (zoomed) compare.applyView(id, CENTERED);
    else compare.applyView(id, { zoomed: true, cx: 0.5, cy: 0.5 });
  }

  function autoAdvance(id: string): void {
    if (!browseView.loupeAutoAdvance) return;
    if (!multi) {
      go(1);
      return;
    }
    if (compare.focusedId === id) advanceFocused(1);
  }

  function rate(value: number | null): void {
    const id = focusedId;
    if (!id) return;
    void rateAsset(id, value).then((ok) => {
      if (ok) autoAdvance(id);
    });
  }

  function favorite(): void {
    const id = focusedId;
    if (!id) return;
    void toggleFavorite(id).then((ok) => {
      if (ok) autoAdvance(id);
    });
  }

  function reject(): void {
    const id = focusedId;
    if (!id) return;
    void toggleReject(id).then((ok) => {
      if (!ok) return;
      const updated = browsing.assets.find((a) => a.id === id);
      if (updated) setTags(id, updated.tags);
      autoAdvance(id);
    });
  }

  function openEditor(): void {
    const id = focusedId;
    if (!id) return;
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
        if (multi) return leaveMulti();
        return browseView.closeLoupe();
      case 'Tab':
        if (!multi) break;
        e.preventDefault();
        return compare.focusDelta(e.shiftKey ? -1 : 1);
      case 'ArrowRight':
        e.preventDefault();
        return multi ? compare.focusDelta(1) : go(1);
      case 'ArrowLeft':
        e.preventDefault();
        return multi ? compare.focusDelta(-1) : go(-1);
      case 'ArrowDown':
        if (!multi) break;
        e.preventDefault();
        return compare.focusDelta(cols);
      case 'ArrowUp':
        if (!multi) break;
        e.preventDefault();
        return compare.focusDelta(-cols);
      case 'Backspace':
      case 'Delete':
        if (!multi) break;
        e.preventDefault();
        return dropFocused();
      case 'k':
      case 'K':
        e.preventDefault();
        return multi ? advanceFocused(1) : go(1);
      case 'j':
      case 'J':
        e.preventDefault();
        return multi ? advanceFocused(-1) : go(-1);
      case 'c':
      case 'C':
        e.preventDefault();
        return enterCompare();
      case 'n':
      case 'N':
        e.preventDefault();
        return enterSurvey();
      case 'y':
      case 'Y':
        e.preventDefault();
        compare.syncView = !compare.syncView;
        return;
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
      case 'Enter':
        e.preventDefault();
        if (compare.mode === 'survey') return compare.keepOnly(compare.focusIndex);
        if (multi) return compare.promote(compare.focusIndex);
        return openEditor();
      case 'e':
      case 'E':
        e.preventDefault();
        return openEditor();
    }
    const next = nextRatingFromKey(e.key, rating);
    if (next !== undefined) {
      e.preventDefault();
      rate(next);
    }
  }

  const hasPrev = $derived(!multi && currentId ? browsing.prevOf(currentId) !== null : false);
  const hasNext = $derived(!multi && currentId ? browsing.nextOf(currentId) !== null : false);
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
        path={mdiCompare}
        size={18}
        title="Compare (C)"
        active={compare.mode === 'compare'}
        pressed={compare.mode === 'compare'}
        onclick={enterCompare}
      />
      <ToolbarButton
        path={mdiViewGridOutline}
        size={18}
        title="Survey (N)"
        active={compare.mode === 'survey'}
        pressed={compare.mode === 'survey'}
        onclick={enterSurvey}
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

    <div
      class="flex-1 min-h-0 relative {multi ? 'grid gap-1 p-1' : 'flex'}"
      style={gridStyle}
    >
      {#each panes as id, index (id)}
        <LoupePane
          assetId={id}
          alt={browsing.assets.find((a) => a.id === id)?.originalFileName ?? ''}
          view={compare.viewOf(id)}
          focused={id === focusedId}
          showFocus={multi}
          badge={multi ? index + 1 : undefined}
          onFocus={() => (compare.focusIndex = index)}
          onView={(next, solo) => compare.applyView(id, next, solo)}
          onSize={(size) => (paneMaxEdge = size)}
        />
      {/each}

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

    <Filmstrip
      currentId={focusedId}
      highlightIds={compare.members}
      onSelect={(id) =>
        multi ? compare.setMember(compare.focusIndex, id) : browseView.openLoupe(id)}
      size={72}
      showBadges
    />
  </div>
{/if}
