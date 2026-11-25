<script lang="ts">
  import type { MapMarker } from '#lib/api/map.ts';
  import type { RecentPlace, SavedPlace } from '#lib/api/places.ts';
  import { coordinateLabel } from '#lib/browse/info/photoInfo.ts';
  import { locationName } from '#lib/browse/locationCopy.ts';
  import ActionMenu from '#lib/components/ActionMenu.svelte';
  import MoreActionsButton from '#lib/components/MoreActionsButton.svelte';
  import NameField from '#lib/components/NameField.svelte';
  import { PLACE_ACTIONS, PLACE_MENU, type PlaceAction } from '#lib/geotag/placeMenu.ts';
  import { spotLine } from '#lib/geotag/placeSuggestions.ts';
  import { idsNear, locationOfSpot } from '#lib/geotag/spots.ts';
  import type { MapPlacesHost } from '#lib/map/placeEditor.svelte.ts';
  import { locationClipboard } from '#lib/stores/locationClipboard.svelte.ts';
  import { places } from '#lib/stores/places.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { mdiClockOutline, mdiStar, mdiStarPlusOutline } from '@mdi/js';

  let { markers, host }: { markers: readonly MapMarker[]; host: MapPlacesHost } = $props();

  let renaming = $state<string | null>(null);

  const counts = $derived(
    new Map(places.saved.map((place) => [place.id, idsNear(markers, place).length]))
  );

  function where(spot: RecentPlace | SavedPlace): string {
    return spotLine(spot) || coordinateLabel(spot.lat, spot.lon);
  }

  function line(place: SavedPlace): string {
    const count = counts.get(place.id) ?? 0;
    return `${where(place)} · ${count === 0 ? 'no photos yet' : countOf(count, 'photo')}`;
  }

  function copy(place: SavedPlace): void {
    const location = locationOfSpot(place);
    locationClipboard.copy(location);
    toasts.push('success', `Copied ${place.name || locationName(location)}`);
  }

  function run(action: PlaceAction, place: SavedPlace): void {
    if (action === 'rename') renaming = place.id;
    else if (action === 'move') host.move(place);
    else if (action === 'copy') copy(place);
    else if (action === 'photos') host.photos(place);
    else void places.remove(place);
  }

  async function rename(place: SavedPlace, name: string): Promise<void> {
    const trimmed = name.trim();
    if (trimmed === place.name || (await places.rename(place, trimmed))) renaming = null;
  }
</script>

<section class="flex min-h-0 flex-col pb-2" aria-label="Places">
  <h3 class="px-4 pt-3 pb-1 section-label text-dark/65">Saved</h3>
  <ul class="flex flex-col" aria-label="Saved places">
    {#each places.saved as place (place.id)}
      <li
        class="flex items-center pe-2"
        data-place-row={place.id}
        onpointerenter={() => host.hover(place)}
        onpointerleave={() => host.hover(null)}
      >
        {#if renaming === place.id}
          <div class="flex min-w-0 flex-1 items-center gap-2 px-4 py-1.5">
            <NameField
              initial={place.name}
              ariaLabel="Name of {place.name}"
              onsave={(name) => rename(place, name)}
              oncancel={() => (renaming = null)}
            />
          </div>
        {:else}
          <Button
            variant="ghost"
            color="secondary"
            class="h-13 min-w-0 flex-1 justify-start gap-3 rounded-none ps-4 pe-2 text-left font-normal touch:h-15"
            aria-label="Show {place.name} on the map"
            onclick={() => host.show(place)}
          >
            <Icon icon={mdiStar} size="16px" class="shrink-0 text-primary" aria-hidden />
            <span class="min-w-0 flex-1">
              <span class="block truncate text-sm text-light-950 touch:text-base">
                {place.name}
              </span>
              <span class="block truncate text-caption text-dark/65 tabular-nums">
                {line(place)}
              </span>
            </span>
          </Button>
          <MoreActionsButton
            label="Actions for {place.name}"
            title={place.name}
            subtitle={where(place)}
          >
            {#snippet children(close)}
              <ActionMenu
                groups={PLACE_MENU}
                actions={PLACE_ACTIONS}
                variant="popover"
                onrun={(action) => {
                  close();
                  run(action, place);
                }}
              />
            {/snippet}
          </MoreActionsButton>
        {/if}
      </li>
    {:else}
      <li class="px-4 py-3 text-caption text-dark/65">
        {places.loaded
          ? 'Right-click the map, or use New place, to save a spot you come back to'
          : 'Loading…'}
      </li>
    {/each}
  </ul>
  {#if places.freshRecent.length > 0}
    <div class="flex items-center px-4 pt-4 pb-1">
      <h3 class="section-label text-dark/65">Recent</h3>
      <Button
        size="tiny"
        variant="ghost"
        class="ms-auto touch:h-9"
        onclick={() => void places.clearRecent()}
      >
        Clear
      </Button>
    </div>
    <ul class="flex flex-col" aria-label="Recent places">
      {#each places.freshRecent as spot (spot.id)}
        <li class="flex items-center pe-2">
          <Button
            variant="ghost"
            color="secondary"
            class="h-13 min-w-0 flex-1 justify-start gap-3 rounded-none ps-4 pe-2 text-left font-normal touch:h-15"
            aria-label="Show {where(spot)} on the map"
            onclick={() => host.goTo(spot)}
          >
            <Icon icon={mdiClockOutline} size="16px" class="shrink-0 text-dark/65" aria-hidden />
            <span class="min-w-0 flex-1 truncate text-sm text-light-950 touch:text-base">
              {where(spot)}
            </span>
          </Button>
          <IconButton
            size="small"
            variant="ghost"
            color="secondary"
            shape="round"
            class="touch:size-11"
            icon={mdiStarPlusOutline}
            title="Save as a place"
            aria-label="Save {where(spot)} as a place"
            onclick={() => host.create(spot, spot.city ?? '')}
          />
        </li>
      {/each}
    </ul>
  {/if}
</section>
