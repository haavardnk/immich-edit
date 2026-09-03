<script lang="ts">
  import { onDestroy } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseView } from '$lib/stores/browseView.svelte';
  import { compare, CENTERED, type CompareMode } from '$lib/stores/compare.svelte';
  import { selection } from '$lib/stores/selection.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { rateAsset, toggleFavorite, toggleReject, clearFlags } from '$lib/cull';
  import { persistedPreviewUrl } from '$lib/api/preview';
  import { getAsset } from '$lib/api/assets';
  import { addTagToAsset, removeTagFromAsset, upsertTags } from '$lib/api/tags';
  import { toasts } from '$lib/stores/toasts.svelte';
  import { metadataConsent } from '$lib/stores/metadataConsent.svelte';
  import { isRejected } from '$lib/reject';
  import { copyIndex, isCopy } from '$lib/assetKey';
  import { multiMembers, type MultiMode } from '$lib/compareEntry';
  import { putBounded } from '$lib/utils/boundedRecord';
  import { cachedFaceData, loadFaceData } from '$lib/stores/zoomTargets';
  import {
    faceTargets,
    nextTargetIndex,
    sharpestPoint,
    type ZoomTarget
  } from '$lib/utils/zoomTarget';
  import { editorHref } from '$lib/editorNavigation';
  import type { TagRef } from '$lib/types/asset';
  import Filmstrip from '$lib/components/shell/Filmstrip.svelte';
  import LoupeActionRail from '$lib/components/browse/LoupeActionRail.svelte';
  import LoupePane from '$lib/components/browse/LoupePane.svelte';
  import LoupeToolbar from '$lib/components/browse/LoupeToolbar.svelte';
  import { nextRatingFromKey } from '$lib/ratingShortcuts';
  import { hint, matchKeybind, isRadioGroupTarget, type KeybindContext } from '$lib/keybinds';
  import { clampZoom, writeZoomLevel } from '$lib/utils/zoomLevel';
  import { IconButton } from '@immich/ui';
  import { mdiChevronLeft, mdiChevronRight, mdiFullscreenExit } from '@mdi/js';

  const MAX_EDGE = 2560;

  let paneMaxEdge = $state(MAX_EDGE);

  const currentId = $derived(browseView.loupeId);
  const multi = $derived(compare.mode !== 'single');
  const contexts = $derived<KeybindContext[]>(
    compare.mode === 'single' ? ['loupe', 'global'] : [compare.mode, 'global']
  );
  const panes = $derived(multi ? compare.members : currentId ? [currentId] : []);
  const focusedId = $derived(multi ? compare.focusedId : currentId);
  const asset = $derived(
    focusedId ? (browsing.assets.find((a) => a.id === focusedId) ?? null) : null
  );
  const rating = $derived(asset?.exifInfo?.rating ?? 0);
  const rejected = $derived(asset ? isRejected(asset) : false);
  const copyBadge = $derived(
    asset && isCopy(asset.id) ? (asset.copyLabel ?? `Copy ${copyIndex(asset.id)}`) : null
  );
  const exif = $derived(asset?.exifInfo ?? null);
  const paneView = $derived(focusedId ? compare.viewOf(focusedId) : CENTERED);
  const cols = $derived(panes.length <= 4 ? 2 : 3);
  const gridStyle = $derived(
    multi ? `grid-template-columns: repeat(${Math.min(cols, panes.length)}, minmax(0, 1fr));` : ''
  );
  const moreActive = $derived(browseView.loupeAutoAdvance || ui.clipWarn);

  let tagCache = $state<Record<string, TagRef[]>>({});
  let tagOrder = $state<string[]>([]);
  let fitZooms = $state<Record<string, number>>({});
  let paneImages = $state<Record<string, HTMLImageElement>>({});
  let paneImageOrder = $state<string[]>([]);
  let targetIndex = $state<number | null>(null);
  const currentTags = $derived(focusedId ? (tagCache[focusedId] ?? []) : []);
  const focusedFitZoom = $derived(focusedId ? (fitZooms[focusedId] ?? 100) : 100);
  const zoom = $derived(paneView.zoom ?? focusedFitZoom);
  const zoomed = $derived(paneView.zoom !== null && paneView.zoom > focusedFitZoom);

  onDestroy(() => {
    ui.fullscreen = false;
  });

  $effect(() => {
    const id = focusedId;
    targetIndex = null;
    if (id) void loadFaceData(id);
  });

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
        img.src = persistedPreviewUrl(neighbor.id, paneMaxEdge, ui.clipWarn);
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

  function enterMulti(mode: MultiMode): void {
    if (multi) {
      leaveMulti();
      return;
    }
    const members = multiMembers(
      mode,
      browsing.assets.map((a) => a.id),
      selection.selected,
      currentId
    );
    if (members.length < 2) {
      toasts.push('info', `${mode} needs two photos`);
      return;
    }
    compare.enter(mode, members);
  }

  function selectViewMode(mode: CompareMode): void {
    if (mode === compare.mode) return;
    if (mode === 'single') {
      leaveMulti();
      return;
    }
    if (!multi) {
      enterMulti(mode);
      return;
    }
    const focusedId = compare.focusedId;
    const orderedIds = browsing.assets.map((asset) => asset.id);
    const members =
      mode === 'survey'
        ? multiMembers(mode, orderedIds, selection.selected, focusedId)
        : [focusedId, ...compare.members.filter((id) => id !== focusedId)].filter(
            (id): id is string => id !== null
          );
    if (members.length < 2) return;
    compare.enter(mode, members.slice(0, mode === 'compare' ? 2 : undefined));
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

  function togglePane(id: string): void {
    const at = compare.members.indexOf(id);
    if (at < 0) return compare.addMember(id);
    if (compare.members.length > 2) return compare.drop(at);
    browseView.openLoupe(compare.members.find((member) => member !== id) ?? id);
  }

  function pickFromStrip(id: string, additive: boolean): void {
    if (!additive) {
      if (multi) compare.setMember(compare.focusIndex, id);
      else browseView.openLoupe(id);
      return;
    }
    if (multi) return togglePane(id);
    if (currentId && id !== currentId) compare.enter('compare', [currentId, id], 1);
  }

  function toggleZoom(): void {
    const id = focusedId;
    if (!id) return;
    if (cachedFaceData(id) === null) {
      void loadFaceData(id).then(() => stepZoomTarget(id));
      return;
    }
    stepZoomTarget(id);
  }

  function zoomTargetsFor(id: string): ZoomTarget[] {
    const data = cachedFaceData(id);
    const faces = data?.savedView ? faceTargets(data.faces, data.savedView) : [];
    if (faces.length > 0) return faces;
    const image = paneImages[id];
    return [(image ? sharpestPoint(image) : null) ?? { u: 0.5, v: 0.5 }];
  }

  function stepZoomTarget(id: string): void {
    if (id !== focusedId) return;
    const targets = zoomTargetsFor(id);
    const next = nextTargetIndex(zoomed ? targetIndex : null, targets.length);
    targetIndex = next;
    const target = next === null ? null : targets[next];
    if (!target) {
      compare.applyView(id, CENTERED);
      return;
    }
    compare.applyView(id, { zoom: ui.zoomLevel, cx: target.u, cy: target.v });
  }

  function setZoom(value: number): void {
    const id = focusedId;
    if (!id) return;
    const nextZoom = clampZoom(value, focusedFitZoom);
    const view = compare.viewOf(id);
    compare.applyView(
      id,
      nextZoom > focusedFitZoom ? { ...view, zoom: nextZoom } : { ...CENTERED, zoom: nextZoom }
    );
    if (nextZoom <= focusedFitZoom) return;
    ui.zoomLevel = nextZoom;
    writeZoomLevel(nextZoom);
  }

  function fitZoom(): void {
    const id = focusedId;
    if (!id) return;
    compare.applyView(id, CENTERED);
  }

  function setFitZoom(id: string, value: number): void {
    if (fitZooms[id] === value) return;
    fitZooms = { ...fitZooms, [id]: value };
  }

  function setPaneImage(id: string, element: HTMLImageElement): void {
    if (paneImages[id] === element) return;
    const next = putBounded(paneImages, paneImageOrder, id, element, 9);
    paneImages = next.record;
    paneImageOrder = next.order;
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
    browseView.leaveLoupeForEditor(id);
    void goto(editorHref(id, `${page.url.pathname}${page.url.search}`));
  }

  function unflag(): void {
    const id = focusedId;
    if (!id) return;
    void clearFlags(id).then((ok) => {
      if (!ok) return;
      const updated = browsing.assets.find((a) => a.id === id);
      if (updated) setTags(id, updated.tags);
    });
  }

  function onKeydown(e: KeyboardEvent): void {
    if (!currentId) return;
    if (e.key === 'Escape' && ui.fullscreen) {
      e.preventDefault();
      ui.toggleFullscreen();
      return;
    }
    const el = document.activeElement;
    if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT')) {
      return;
    }
    if (isRadioGroupTarget(e)) return;

    const bind = matchKeybind(e, contexts);
    if (!bind) return;

    switch (bind) {
      case 'help':
        e.preventDefault();
        return ui.toggleKeybindsHelp();
      case 'loupeClose':
        e.preventDefault();
        return browseView.closeLoupe();
      case 'compareExit':
      case 'surveyExit':
        e.preventDefault();
        return leaveMulti();
      case 'backToGrid':
        e.preventDefault();
        if (multi) leaveMulti();
        return browseView.closeLoupe();
      case 'loupeNav':
        e.preventDefault();
        return go(e.key === 'ArrowRight' ? 1 : -1);
      case 'compareFocus':
        e.preventDefault();
        return compare.focusDelta(e.key === 'ArrowRight' ? 1 : -1);
      case 'surveyFocus':
        e.preventDefault();
        if (e.key === 'ArrowRight') return compare.focusDelta(1);
        if (e.key === 'ArrowLeft') return compare.focusDelta(-1);
        return compare.focusDelta(e.key === 'ArrowDown' ? cols : -cols);
      case 'paneFocusCycle':
        e.preventDefault();
        return compare.focusDelta(e.shiftKey ? -1 : 1);
      case 'paneSwap':
        e.preventDefault();
        return advanceFocused(e.key === 'ArrowRight' ? 1 : -1);
      case 'paneDrop':
        e.preventDefault();
        return dropFocused();
      case 'paneSync':
        e.preventDefault();
        compare.syncView = !compare.syncView;
        return;
      case 'panePromote':
        e.preventDefault();
        return compare.promote(compare.focusIndex);
      case 'surveyKeep':
        e.preventDefault();
        return compare.keepOnly(compare.focusIndex);
      case 'enterCompare':
        e.preventDefault();
        return enterMulti('compare');
      case 'enterSurvey':
        e.preventDefault();
        return enterMulti('survey');
      case 'openEditor':
      case 'paneOpenEditor':
        e.preventDefault();
        return openEditor();
      case 'favorite':
        e.preventDefault();
        return favorite();
      case 'reject':
        e.preventDefault();
        return reject();
      case 'unflag':
        e.preventDefault();
        return unflag();
      case 'zoomToggle':
        e.preventDefault();
        return toggleZoom();
      case 'toggleInfo':
        e.preventDefault();
        browseView.loupeInfoOpen = !browseView.loupeInfoOpen;
        return;
      case 'toggleTags':
        e.preventDefault();
        browseView.loupeTagsOpen = !browseView.loupeTagsOpen;
        return;
      case 'clipWarn':
        e.preventDefault();
        ui.toggleClipWarn();
        return;
      case 'fullscreen':
        e.preventDefault();
        ui.toggleFullscreen();
        return;
      case 'rate': {
        const next = nextRatingFromKey(e.key, rating);
        if (next === undefined) return;
        e.preventDefault();
        return rate(next);
      }
    }
  }

  const hasPrev = $derived(!multi && currentId ? browsing.prevOf(currentId) !== null : false);
  const hasNext = $derived(!multi && currentId ? browsing.nextOf(currentId) !== null : false);
</script>

<svelte:window onkeydown={onKeydown} />

{#if asset}
  <div class="fixed inset-0 z-40 flex min-w-0 flex-col bg-image-canvas">
    {#if !ui.fullscreen}
      <LoupeToolbar
        filename={asset.originalFileName}
        {copyBadge}
        {multi}
        {moreActive}
        onSelectViewMode={selectViewMode}
        onOpenEditor={openEditor}
      />
    {/if}

    <div
      class="flex-1 min-h-0 relative {multi ? 'grid gap-1.5 bg-neutral-950 p-1.5' : 'flex'}"
      style={gridStyle}
    >
      {#each panes as id, index (id)}
        {@const paneAsset = browsing.assets.find((item) => item.id === id)}
        <LoupePane
          assetId={id}
          alt={paneAsset?.originalFileName ?? ''}
          view={compare.viewOf(id)}
          focused={id === focusedId}
          showFocus={multi}
          badge={multi ? index + 1 : undefined}
          onFocus={() => (compare.focusIndex = index)}
          onView={(next, solo) => {
            targetIndex = null;
            compare.applyView(id, next, solo);
          }}
          onSize={(size) => (paneMaxEdge = size)}
          sourceLong={Math.max(
            paneAsset?.exifInfo?.exifImageWidth ?? 0,
            paneAsset?.exifInfo?.exifImageHeight ?? 0
          )}
          onFitZoom={(value) => setFitZoom(id, value)}
          onImage={(element) => setPaneImage(id, element)}
        />
      {/each}

      {#if hasPrev && !ui.fullscreen}
        <IconButton
          type="button"
          size="medium"
          variant="ghost"
          color="secondary"
          shape="round"
          class="absolute top-1/2 left-2 -translate-y-1/2 bg-black/40 text-white hover:bg-black/70"
          icon={mdiChevronLeft}
          title="Previous"
          aria-label="Previous"
          onclick={() => go(-1)}
        />
      {/if}
      {#if hasNext && !ui.fullscreen}
        <IconButton
          type="button"
          size="medium"
          variant="ghost"
          color="secondary"
          shape="round"
          class="absolute top-1/2 right-2 -translate-y-1/2 bg-black/40 text-white hover:bg-black/70"
          icon={mdiChevronRight}
          title="Next"
          aria-label="Next"
          onclick={() => go(1)}
        />
      {/if}

      {#if browseView.loupeInfoOpen && exif && !ui.fullscreen}
        <div
          class="absolute top-2 right-2 flex w-56 flex-col gap-1 rounded-lg border border-gray-700 bg-gray-900/95 p-3 text-[11px] text-white/80 shadow-xl"
        >
          {#if exif.make || exif.model}
            <span>{[exif.make, exif.model].filter(Boolean).join(' ')}</span>
          {/if}
          {#if exif.lensModel}<span class="text-muted">{exif.lensModel}</span>{/if}
          <div class="flex flex-wrap gap-x-3 gap-y-0.5 text-muted">
            {#if exif.fNumber}<span>ƒ/{exif.fNumber}</span>{/if}
            {#if exif.exposureTime}<span>{exif.exposureTime}s</span>{/if}
            {#if exif.iso}<span>ISO {exif.iso}</span>{/if}
            {#if exif.focalLength}<span>{exif.focalLength}mm</span>{/if}
          </div>
          {#if exif.exifImageWidth && exif.exifImageHeight}
            <span class="text-muted">{exif.exifImageWidth} × {exif.exifImageHeight}</span>
          {/if}
          {#if exif.dateTimeOriginal}
            <span class="text-muted">{exif.dateTimeOriginal}</span>
          {/if}
        </div>
      {/if}

      {#if ui.fullscreen}
        <IconButton
          size="small"
          variant="filled"
          color="primary"
          shape="round"
          class="fixed top-3 right-3 z-40 shadow-lg"
          icon={mdiFullscreenExit}
          title={hint('Exit fullscreen', 'fullscreen')}
          aria-label={hint('Exit fullscreen', 'fullscreen')}
          onclick={ui.toggleFullscreen}
        />
      {/if}
    </div>

    {#if !ui.fullscreen}
      <LoupeActionRail
        {rating}
        isFavorite={asset.isFavorite}
        {rejected}
        tags={currentTags}
        {multi}
        paneCount={panes.length}
        {zoom}
        fitZoom={focusedFitZoom}
        fitMode={paneView.zoom === null}
        onRate={rate}
        onFavorite={favorite}
        onReject={reject}
        onAddTag={addTag}
        onRemoveTag={removeTag}
        onCreateTag={createAndAddTag}
        onZoom={setZoom}
        onFit={fitZoom}
      />

      <Filmstrip
        currentId={focusedId}
        highlightIds={compare.members}
        onSelect={pickFromStrip}
        resizable
        size={72}
        showBadges
        collapsed={ui.loupeFilmstripCollapsed}
      />
    {/if}
  </div>
{/if}
