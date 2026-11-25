<script lang="ts">
  import type { MapMarker } from '#lib/api/map.ts';
  import type { SavedPlace } from '#lib/api/places.ts';
  import { coordinateLabel } from '#lib/browse/info/photoInfo.ts';
  import { placeAt } from '#lib/browse/locationOps.ts';
  import { locationName } from '#lib/browse/locationCopy.ts';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import NameField from '#lib/components/NameField.svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import { PLACE_ACTIONS } from '#lib/geotag/placeMenu.ts';
  import { spotLine } from '#lib/geotag/placeSuggestions.ts';
  import { awayFrom, locationOfSpot } from '#lib/geotag/spots.ts';
  import { locationClipboard } from '#lib/stores/locationClipboard.svelte.ts';
  import { places } from '#lib/stores/places.svelte.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { mdiMapMarkerOutline } from '@mdi/js';

  let {
    place,
    markers,
    onMove,
    onClose
  }: {
    place: SavedPlace;
    markers: readonly MapMarker[];
    onMove: (place: SavedPlace) => void;
    onClose: () => void;
  } = $props();

  let renaming = $state(false);

  const line = $derived(
    [spotLine(place), coordinateLabel(place.lat, place.lon)].filter(Boolean).join(' · ')
  );
  const movable = $derived(awayFrom(selection.selected, markers, place));

  function placeHere(): void {
    const ids = movable;
    const { lat, lon, place: name } = locationOfSpot(place);
    onClose();
    void placeAt(ids, lat, lon, name);
  }

  function copy(): void {
    const location = locationOfSpot(place);
    locationClipboard.copy(location);
    toasts.push('success', `Copied ${place.name || locationName(location)}`);
    onClose();
  }

  async function rename(name: string): Promise<void> {
    const trimmed = name.trim();
    if (trimmed === place.name || (await places.rename(place, trimmed))) onClose();
  }
</script>

<ActionSheet
  open
  onOpenChange={(open) => {
    if (!open) onClose();
  }}
  title={place.name}
  caption
  subtitle={line}
>
  {#if renaming}
    <div class="px-4 py-3">
      <NameField
        initial={place.name}
        ariaLabel="Name of {place.name}"
        onsave={rename}
        oncancel={() => (renaming = false)}
      />
    </div>
  {:else}
    {#if movable.length > 0}
      <MenuRow
        icon={mdiMapMarkerOutline}
        label="Place {countOf(movable.length, 'selected photo')} here"
        onclick={placeHere}
      />
    {/if}
    <MenuRow icon={PLACE_ACTIONS.copy.icon} label={PLACE_ACTIONS.copy.label} onclick={copy} />
    <MenuRow
      icon={PLACE_ACTIONS.rename.icon}
      label={PLACE_ACTIONS.rename.label}
      onclick={() => (renaming = true)}
    />
    <MenuRow
      icon={PLACE_ACTIONS.move.icon}
      label="Move"
      onclick={() => {
        onClose();
        onMove(place);
      }}
    />
    <MenuRow
      icon={PLACE_ACTIONS.remove.icon}
      label={PLACE_ACTIONS.remove.label}
      danger
      onclick={() => {
        onClose();
        void places.remove(place);
      }}
    />
  {/if}
</ActionSheet>
