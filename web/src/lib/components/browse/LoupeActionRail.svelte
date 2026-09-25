<script lang="ts">
  import { browseView } from '$lib/stores/browseView.svelte';
  import { compare } from '$lib/stores/compare.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import type { TagRef } from '$lib/types/asset';
  import TagPicker from '$lib/components/TagPicker.svelte';
  import ZoomPopover from '$lib/components/ZoomPopover.svelte';
  import StarRating from '$lib/components/StarRating.svelte';
  import FavoriteButton from '$lib/components/FavoriteButton.svelte';
  import RejectButton from '$lib/components/RejectButton.svelte';
  import LabelPicker from '$lib/components/LabelPicker.svelte';
  import type { LabelColor } from '$lib/labels';
  import { hint } from '$lib/keybinds';
  import { Icon, IconButton } from '@immich/ui';
  import {
    mdiArrowCollapseLeft,
    mdiCheckboxMarkedCircleOutline,
    mdiClose,
    mdiCompare,
    mdiFilmstrip,
    mdiLink,
    mdiLinkOff,
    mdiViewGridOutline
  } from '@mdi/js';

  type Props = {
    rating: number;
    isFavorite: boolean;
    rejected: boolean;
    label: LabelColor | null;
    tags: TagRef[];
    multi: boolean;
    position: number;
    count: number;
    canDrop: boolean;
    zoom: number;
    fitZoom: number;
    fitMode: boolean;
    onRate: (value: number | null) => void;
    onFavorite: () => void;
    onReject: () => void;
    onLabel: (color: LabelColor | null) => void;
    onAddTag: (tag: TagRef) => Promise<void>;
    onRemoveTag: (tagId: string) => Promise<void>;
    onCreateTag: (value: string) => Promise<TagRef | null>;
    onZoom: (value: number) => void;
    onFit: () => void;
    onDrop: () => void;
  };

  let {
    rating,
    isFavorite,
    rejected,
    label,
    tags,
    multi,
    position,
    count,
    canDrop,
    zoom,
    fitZoom,
    fitMode,
    onRate,
    onFavorite,
    onReject,
    onLabel,
    onAddTag,
    onRemoveTag,
    onCreateTag,
    onZoom,
    onFit,
    onDrop
  }: Props = $props();
</script>

<nav
  aria-label="Photo actions"
  class="relative grid h-9 shrink-0 grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center border-t border-hairline bg-editor-chrome px-2"
>
  <div class="min-w-0 overflow-hidden">
    <div class="flex min-w-0 items-center gap-2">
      <div class="shrink-0">
        <StarRating {rating} size={16} onchange={onRate} />
      </div>
      <FavoriteButton {isFavorite} ontoggle={onFavorite} />
      <RejectButton isRejected={rejected} ontoggle={onReject} />
      <LabelPicker {label} onchange={onLabel} />
      <div class="min-w-0">
        <TagPicker
          {tags}
          open={browseView.loupeTagsOpen}
          onToggle={() => (browseView.loupeTagsOpen = !browseView.loupeTagsOpen)}
          onClose={() => (browseView.loupeTagsOpen = false)}
          onAdd={onAddTag}
          onRemove={onRemoveTag}
          onCreate={onCreateTag}
          anchor="bottom"
        />
      </div>
    </div>
  </div>
  <div class="flex items-center gap-1">
    <div class="flex h-6 items-center gap-1.5 rounded bg-ghost px-2.5 text-[10px] text-white/65">
      {#if multi}
        <Icon
          icon={compare.mode === 'compare' ? mdiCompare : mdiViewGridOutline}
          size="13px"
          class="text-primary"
          aria-hidden="true"
        />
        <span class="font-medium text-white/85">
          {compare.mode === 'compare' ? 'Compare' : 'Survey'}
        </span>
        <span class="h-3 w-px bg-white/15"></span>
      {/if}
      <span class="tabular-nums">{position} of {count}</span>
    </div>
    {#if multi}
      <IconButton
        size="tiny"
        variant="ghost"
        color={compare.syncView ? 'primary' : 'secondary'}
        icon={compare.syncView ? mdiLink : mdiLinkOff}
        title={hint('Sync zoom and pan', 'paneSync')}
        aria-label="Sync zoom and pan"
        aria-pressed={compare.syncView}
        onclick={() => (compare.syncView = !compare.syncView)}
      />
      {#if compare.mode === 'compare'}
        <IconButton
          size="tiny"
          variant="ghost"
          color="secondary"
          icon={mdiArrowCollapseLeft}
          title={hint('Promote to the left', 'panePromote')}
          aria-label="Promote to the left"
          disabled={compare.focusIndex === 0}
          onclick={() => compare.promote(compare.focusIndex)}
        />
      {:else}
        <IconButton
          size="tiny"
          variant="ghost"
          color="secondary"
          icon={mdiCheckboxMarkedCircleOutline}
          title={hint('Keep only this photo', 'surveyKeep')}
          aria-label="Keep only this photo"
          onclick={() => compare.keepOnly(compare.focusIndex)}
        />
      {/if}
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        icon={mdiClose}
        title={hint('Drop this photo', 'paneDrop')}
        aria-label="Drop this photo"
        disabled={!canDrop}
        onclick={onDrop}
      />
    {/if}
  </div>
  <div class="col-start-3 flex min-w-0 items-center justify-end gap-0.5">
    <ZoomPopover
      open={ui.metaPopover === 'zoom'}
      {zoom}
      {fitZoom}
      {fitMode}
      onOpenChange={(value) => (value ? ui.openPopover('zoom') : ui.closePopover())}
      {onZoom}
      {onFit}
    />
    <IconButton
      size="tiny"
      variant="ghost"
      color={!ui.loupeFilmstripCollapsed ? 'primary' : 'secondary'}
      icon={mdiFilmstrip}
      title={hint(
        ui.loupeFilmstripCollapsed ? 'Show filmstrip' : 'Collapse filmstrip',
        'loupeFilmstrip'
      )}
      aria-label={ui.loupeFilmstripCollapsed ? 'Show filmstrip' : 'Collapse filmstrip'}
      aria-pressed={!ui.loupeFilmstripCollapsed}
      onclick={ui.toggleLoupeFilmstrip}
    />
  </div>
</nav>
