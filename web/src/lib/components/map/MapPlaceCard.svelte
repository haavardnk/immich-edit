<script lang="ts">
  import type { MapMarker } from '#lib/api/map.ts';
  import type { SavedPlace } from '#lib/api/places.ts';
  import { placeAt } from '#lib/browse/locationOps.ts';
  import { locationName } from '#lib/browse/locationCopy.ts';
  import { coordinateLabel } from '#lib/browse/info/photoInfo.ts';
  import { spotLine } from '#lib/geotag/placeSuggestions.ts';
  import { awayFrom, locationOfSpot } from '#lib/geotag/spots.ts';
  import NameField from '#lib/components/NameField.svelte';
  import { followPoint, type ScreenPoint } from '#lib/map/followPoint.ts';
  import { places } from '#lib/stores/places.svelte.ts';
  import { locationClipboard } from '#lib/stores/locationClipboard.svelte.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button, Icon, IconButton } from '@immich/ui';
  import {
    mdiArrowAll,
    mdiContentCopy,
    mdiDeleteOutline,
    mdiPencilOutline,
    mdiStar
  } from '@mdi/js';
  import type { Map as MapLibreMap } from 'maplibre-gl';

  let {
    place,
    map,
    markers,
    onMove,
    onClose
  }: {
    place: SavedPlace;
    map: MapLibreMap | null;
    markers: readonly MapMarker[];
    onMove: (place: SavedPlace) => void;
    onClose: () => void;
  } = $props();

  const CARD_SPACE = 150;
  const CARD_HALF = 156;

  let point = $state<ScreenPoint | null>(null);
  let renaming = $state(false);

  const line = $derived(
    [spotLine(place), coordinateLabel(place.lat, place.lon)].filter(Boolean).join(' · ')
  );
  const movable = $derived(awayFrom(selection.selected, markers, place));

  $effect(() => {
    if (!map) return;
    return followPoint(map, place, (next) => (point = next));
  });

  $effect(() => {
    const key = (event: KeyboardEvent): void => {
      if (event.key !== 'Escape' || renaming) return;
      event.preventDefault();
      event.stopPropagation();
      onClose();
    };
    window.addEventListener('keydown', key, true);
    return () => window.removeEventListener('keydown', key, true);
  });

  function copy(): void {
    const location = locationOfSpot(place);
    locationClipboard.copy(location);
    toasts.push('success', `Copied ${place.name || locationName(location)}`);
    onClose();
  }

  async function rename(name: string): Promise<void> {
    const trimmed = name.trim();
    if (trimmed === place.name || (await places.rename(place, trimmed))) renaming = false;
  }

  function remove(): void {
    onClose();
    void places.remove(place);
  }

  function placeHere(): void {
    const ids = movable;
    const { lat, lon, place: name } = locationOfSpot(place);
    onClose();
    void placeAt(ids, lat, lon, name);
  }

  const focus = (card: HTMLElement): void => {
    [...card.querySelectorAll<HTMLButtonElement>('button')].at(-1)?.focus();
  };
</script>

{#snippet action(
  icon: string,
  label: string,
  run: () => void,
  color: 'secondary' | 'danger' = 'secondary'
)}
  <IconButton
    size="small"
    variant="ghost"
    {color}
    class="shrink-0 touch:size-11"
    {icon}
    title={label}
    aria-label={label}
    onclick={run}
  />
{/snippet}

{#if point}
  {@const below = point.y < CARD_SPACE}
  <div
    role="dialog"
    aria-label={place.name}
    class="absolute z-30 w-78 -translate-x-1/2 rounded-xl bg-light-100 p-3 shadow-popover ring-1 ring-hairline"
    class:-translate-y-full={!below}
    style:left="{Math.min(Math.max(CARD_HALF, point.x), point.width - CARD_HALF)}px"
    style:top="{below ? point.y + 24 : point.y - 24}px"
    data-place-card
    {@attach focus}
  >
    {#if renaming}
      <NameField
        initial={place.name}
        ariaLabel="Name of {place.name}"
        onsave={rename}
        oncancel={() => (renaming = false)}
      />
    {:else}
      <div class="flex items-center gap-2">
        <Icon icon={mdiStar} size="16px" class="shrink-0 text-primary" aria-hidden />
        <p class="min-w-0 flex-1 truncate text-sm font-semibold text-light-950">{place.name}</p>
        <IconButton
          size="small"
          variant="ghost"
          color="secondary"
          class="shrink-0 touch:size-11"
          icon={mdiPencilOutline}
          title="Rename"
          aria-label="Rename {place.name}"
          onclick={() => (renaming = true)}
        />
      </div>
    {/if}
    <p class="mt-0.5 truncate text-caption text-dark/65 tabular-nums">{line}</p>
    <div class="mt-3 flex items-center gap-1">
      {@render action(mdiContentCopy, 'Copy location', copy)}
      {@render action(mdiArrowAll, 'Move', () => onMove(place))}
      {@render action(mdiDeleteOutline, 'Remove', remove, 'danger')}
      <span class="flex-1"></span>
      {#if movable.length > 0}
        <Button size="small" class="touch:h-11" onclick={placeHere}>
          Place {countOf(movable.length, 'selected photo')} here
        </Button>
      {/if}
    </div>
  </div>
{/if}
