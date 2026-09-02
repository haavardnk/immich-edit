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
  import { Icon, IconButton } from '@immich/ui';
  import { mdiCompare, mdiViewGridOutline, mdiFilmstrip } from '@mdi/js';

  type Props = {
    rating: number;
    isFavorite: boolean;
    rejected: boolean;
    tags: TagRef[];
    multi: boolean;
    paneCount: number;
    zoom: number;
    nativeZoom: number | null;
    onRate: (value: number | null) => void;
    onFavorite: () => void;
    onReject: () => void;
    onAddTag: (tag: TagRef) => Promise<void>;
    onRemoveTag: (tagId: string) => Promise<void>;
    onCreateTag: (value: string) => Promise<TagRef | null>;
    onZoom: (value: number) => void;
  };

  let {
    rating,
    isFavorite,
    rejected,
    tags,
    multi,
    paneCount,
    zoom,
    nativeZoom,
    onRate,
    onFavorite,
    onReject,
    onAddTag,
    onRemoveTag,
    onCreateTag,
    onZoom
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
  {#if multi}
    <div class="flex h-6 items-center gap-1.5 rounded bg-ghost px-2.5 text-[10px] text-white/65">
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
      <span class="tabular-nums">{compare.focusIndex + 1} of {paneCount}</span>
    </div>
  {/if}
  <div class="col-start-3 flex min-w-0 items-center justify-end gap-0.5">
    <ZoomPopover
      open={ui.metaPopover === 'zoom'}
      {zoom}
      {nativeZoom}
      onOpenChange={(value) => (value ? ui.openPopover('zoom') : ui.closePopover())}
      {onZoom}
      onFit={() => onZoom(100)}
    />
    <IconButton
      size="tiny"
      variant="ghost"
      color={!ui.loupeFilmstripCollapsed ? 'primary' : 'secondary'}
      icon={mdiFilmstrip}
      title={ui.loupeFilmstripCollapsed ? 'Show filmstrip' : 'Collapse filmstrip'}
      aria-label={ui.loupeFilmstripCollapsed ? 'Show filmstrip' : 'Collapse filmstrip'}
      aria-pressed={!ui.loupeFilmstripCollapsed}
      onclick={ui.toggleLoupeFilmstrip}
    />
  </div>
</nav>
