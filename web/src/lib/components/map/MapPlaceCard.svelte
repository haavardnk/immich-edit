<script lang="ts">
  import type { MapMarker } from '#lib/api/map.ts';
  import type { SavedPlace } from '#lib/api/places.ts';
  import { placeAt } from '#lib/browse/locationOps.ts';
  import { locationName } from '#lib/browse/locationCopy.ts';
  import { coordinateLabel } from '#lib/browse/info/photoInfo.ts';
  import { spotLine } from '#lib/geotag/placeSuggestions.ts';
  import { awayFrom, locationOfSpot } from '#lib/geotag/spots.ts';
  import { followPoint, type ScreenPoint } from '#lib/map/followPoint.ts';
  import { locationClipboard } from '#lib/stores/locationClipboard.svelte.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiStar } from '@mdi/js';
  import type { Map as MapLibreMap } from 'maplibre-gl';

  let {
    place,
    map,
    markers,
    onClose
  }: {
    place: SavedPlace;
    map: MapLibreMap | null;
    markers: readonly MapMarker[];
    onClose: () => void;
  } = $props();

  const CARD_SPACE = 150;
  const CARD_HALF = 136;

  let point = $state<ScreenPoint | null>(null);

  const line = $derived(spotLine(place) || coordinateLabel(place.lat, place.lon));
  const movable = $derived(awayFrom(selection.selected, markers, place));

  $effect(() => {
    if (!map) return;
    return followPoint(map, place, (next) => (point = next));
  });

  $effect(() => {
    const key = (event: KeyboardEvent): void => {
      if (event.key !== 'Escape') return;
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

{#if point}
  {@const below = point.y < CARD_SPACE}
  <div
    role="dialog"
    aria-label={place.name}
    class="absolute z-30 w-64 -translate-x-1/2 rounded-xl bg-light-100 p-3 shadow-popover ring-1 ring-hairline"
    class:-translate-y-full={!below}
    style:left="{Math.min(Math.max(CARD_HALF, point.x), point.width - CARD_HALF)}px"
    style:top="{below ? point.y + 24 : point.y - 24}px"
    data-place-card
    {@attach focus}
  >
    <p class="flex items-center gap-2 text-sm font-semibold text-light-950">
      <Icon icon={mdiStar} size="16px" class="shrink-0 text-primary" aria-hidden />
      <span class="truncate">{place.name}</span>
    </p>
    <p class="mt-0.5 truncate text-caption text-dark/65">{line}</p>
    <div class="mt-3 flex flex-wrap justify-end gap-2">
      <Button size="small" variant="ghost" color="secondary" class="touch:h-11" onclick={copy}>
        Copy location
      </Button>
      {#if movable.length > 0}
        <Button size="small" class="touch:h-11" onclick={placeHere}>
          Place {countOf(movable.length, 'selected photo')} here
        </Button>
      {/if}
    </div>
  </div>
{/if}
