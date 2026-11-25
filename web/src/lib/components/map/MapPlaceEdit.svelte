<script lang="ts">
  import { coordinateLabel } from '#lib/browse/info/photoInfo.ts';
  import TextInput from '#lib/components/TextInput.svelte';
  import { spotLine } from '#lib/geotag/placeSuggestions.ts';
  import { followPoint, type ScreenPoint } from '#lib/map/followPoint.ts';
  import { PLACE_ZOOM, type PlaceEditor } from '#lib/map/placeEditor.svelte.ts';
  import type { PlaceStars } from '#lib/map/placeStars.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiStar, mdiStarOutline } from '@mdi/js';
  import type { Map as MapLibreMap } from 'maplibre-gl';
  import { untrack } from 'svelte';

  let {
    editor,
    map,
    stars,
    touch
  }: {
    editor: PlaceEditor;
    map: MapLibreMap | null;
    stars: PlaceStars | null;
    touch: boolean;
  } = $props();

  const CARD_HALF = 156;

  let point = $state<ScreenPoint | null>(null);
  let origin = $state<ScreenPoint | null>(null);

  const edit = $derived(editor.edit);
  const kind = $derived(edit?.kind ?? null);
  const movingId = $derived(edit?.kind === 'move' ? edit.place.id : null);
  const title = $derived(edit?.kind === 'move' ? edit.place.name : 'New place');
  const line = $derived(
    edit
      ? [editor.where ? spotLine(editor.where) : '', coordinateLabel(edit.at.lat, edit.at.lon)]
          .filter(Boolean)
          .join(' · ')
      : ''
  );

  $effect(() => {
    const layer = stars;
    if (!layer || touch || !kind) return;
    if (kind === 'move' && movingId) {
      const id = movingId;
      untrack(() => layer.edit(id, editor.place));
      return () => untrack(() => layer.edit(null));
    }
    const at = untrack(() => editor.edit?.at ?? null);
    untrack(() => layer.ghost(at, editor.place));
    return () => untrack(() => layer.ghost(null));
  });

  $effect(() => {
    const layer = stars;
    if (!layer || !touch) return;
    const id = movingId;
    untrack(() => layer.hide(id));
    return () => untrack(() => layer.hide(null));
  });

  $effect(() => {
    const target = map;
    if (!target || !touch || !kind) return;
    const at = untrack(() => editor.edit?.at);
    if (at) {
      untrack(() =>
        target.jumpTo({
          center: [at.lon, at.lat],
          zoom: Math.max(target.getZoom(), PLACE_ZOOM)
        })
      );
    }
    const settle = (): void => {
      const center = target.getCenter();
      editor.place({ lat: center.lat, lon: center.lng });
    };
    target.on('moveend', settle);
    return () => target.off('moveend', settle);
  });

  $effect(() => {
    const target = map;
    const at = edit?.at;
    if (!target || touch || kind !== 'create' || !at) return;
    return followPoint(target, at, (next) => (point = next));
  });

  $effect(() => {
    const target = map;
    const from = edit?.kind === 'move' ? edit.place : null;
    origin = null;
    if (!target || touch || !from) return;
    return followPoint(target, from, (next) => (origin = next));
  });

  $effect(() => {
    if (!kind) return;
    const key = (event: KeyboardEvent): void => {
      if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        editor.cancel();
      } else if (event.key === 'Enter' && kind === 'move') {
        event.preventDefault();
        void editor.done();
      }
    };
    window.addEventListener('keydown', key, true);
    return () => window.removeEventListener('keydown', key, true);
  });

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    void editor.done();
  }
</script>

{#snippet nameForm(compact: boolean)}
  <form class="flex flex-col gap-2" onsubmit={submit}>
    <TextInput label="Place name" bind:value={editor.name} maxlength={80} autofocus />
    <div class="flex items-center gap-2">
      {#if compact}
        <span class="min-w-0 flex-1 text-caption text-dark/65">
          Shows up in search and the Location menu
        </span>
      {/if}
      <Button
        size={compact ? 'small' : 'large'}
        variant="ghost"
        color="secondary"
        class={compact ? '' : 'flex-1'}
        onclick={() => editor.cancel()}
      >
        Cancel
      </Button>
      <Button
        type="submit"
        size={compact ? 'small' : 'large'}
        class={compact ? '' : 'flex-1'}
        loading={editor.busy}
        disabled={!editor.name.trim()}
      >
        Save
      </Button>
    </div>
  </form>
{/snippet}

{#if edit}
  {#if origin}
    <span
      class="pointer-events-none absolute z-10 size-8 -translate-1/2 rounded-full border-2 border-dashed border-light-950/70"
      style:left="{origin.x}px"
      style:top="{origin.y}px"
      data-place-origin
    ></span>
  {/if}
  {#if touch}
    <div
      class="pointer-events-none absolute top-1/2 left-1/2 z-20 grid size-16 -translate-1/2 place-items-center rounded-full ring-2 ring-light-950/85"
      data-place-mark
    >
      <span
        class="grid size-8 place-items-center rounded-full bg-light-100 shadow-lg ring-2 ring-primary"
      >
        <Icon
          icon={kind === 'move' ? mdiStar : mdiStarOutline}
          size="16px"
          class="text-primary"
          aria-hidden
        />
      </span>
    </div>
    <p
      class="pointer-events-none absolute top-[calc(50%+3rem)] left-1/2 z-20 max-w-[90%] -translate-x-1/2 truncate rounded-full bg-light-100 px-3.5 py-2 text-sm text-light-950 shadow-popover tabular-nums"
      role="status"
    >
      {line}
    </p>
  {/if}
  {#if kind === 'move' || touch}
    <div
      class="absolute top-3 left-1/2 z-30 flex max-w-[calc(100%-1.5rem)] -translate-x-1/2 items-center gap-3 rounded-xl bg-light-100 py-1.5 ps-3.5 pe-1.5 text-sm text-light-950 shadow-popover ring-1 ring-hairline"
      role="toolbar"
      aria-label={kind === 'move' ? `Move ${title}` : 'New place'}
      data-place-edit
    >
      <Icon icon={mdiStar} size="16px" class="shrink-0 text-primary" aria-hidden />
      <span class="min-w-0 truncate touch:leading-tight">
        {kind === 'move' ? `Moving ${title}` : 'New place'}
        <span class="text-dark/65 touch:block touch:text-caption">
          <span class="touch:hidden">·</span>
          {touch ? 'Pan the map under the mark' : 'drag the star, then Done'}
        </span>
      </span>
      {#if kind === 'move'}
        <Button
          size="small"
          variant="ghost"
          color="secondary"
          class="touch:h-11"
          onclick={() => editor.cancel()}
        >
          Cancel
        </Button>
        <Button
          size="small"
          class="touch:h-11"
          loading={editor.busy}
          onclick={() => void editor.done()}
        >
          Done
        </Button>
      {/if}
    </div>
  {/if}
  {#if kind === 'create' && touch}
    <div
      class="absolute inset-x-0 bottom-0 z-30 rounded-t-2xl bg-light px-4 pt-4 pb-[calc(1rem+var(--safe-bottom,0px))] shadow-popover"
      role="dialog"
      aria-label="New place"
    >
      {@render nameForm(false)}
    </div>
  {:else if kind === 'create' && point}
    {@const below = point.y < 220}
    <div
      class="absolute z-30 w-78 -translate-x-1/2 rounded-xl bg-light-100 p-3 shadow-popover ring-1 ring-hairline"
      class:-translate-y-full={!below}
      style:left="{Math.min(Math.max(CARD_HALF, point.x), point.width - CARD_HALF)}px"
      style:top="{below ? point.y + 26 : point.y - 26}px"
      role="dialog"
      aria-label="New place"
      data-place-new
    >
      <p class="text-sm font-semibold text-light-950">New place</p>
      <p class="mb-2 truncate text-caption text-dark/65 tabular-nums">{line}</p>
      {@render nameForm(true)}
    </div>
  {/if}
{/if}
