import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
import type { BulkMenuAction } from '#lib/browse/bulk/bulkMenu.ts';
import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
import { copyLocationOf, locationName, pasteLocationTo } from '#lib/browse/locationCopy.ts';
import { locationOf, placeAt } from '#lib/browse/locationOps.ts';
import { matchTrackFor } from '#lib/browse/trackEntry.ts';
import type { RecentPlace, SavedPlace } from '#lib/api/places.ts';
import { locationOfSpot } from '#lib/geotag/spots.ts';
import { locationClipboard } from '#lib/stores/locationClipboard.svelte.ts';
import { locationPicker } from '#lib/stores/locationPicker.svelte.ts';
import { places } from '#lib/stores/places.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import {
  mdiContentCopy,
  mdiContentPaste,
  mdiHistory,
  mdiMapMarkerOutline,
  mdiMapMarkerPath,
  mdiStar
} from '@mdi/js';

const MENU_SAVED = 5;
const MENU_RECENT = 3;

function spotDetail(spot: RecentPlace | SavedPlace, label: string): string | null {
  return [spot.city, spot.state, spot.country].find((part) => part && part !== label) ?? null;
}

function placeSelection(spot: RecentPlace | SavedPlace): Promise<boolean> {
  const { lat, lon, place } = locationOfSpot(spot);
  return placeAt([...selection.selected], lat, lon, place);
}

class BulkLocation {
  readonly #blocked = $derived(bulkScope.count === 0 || bulkActions.metaBusy);
  readonly #copied = $derived(locationClipboard.copied);

  readonly choose: BulkMenuAction = $derived({
    label: 'Choose on map…',
    icon: mdiMapMarkerOutline,
    bind: null,
    detail: null,
    disabled: this.#blocked,
    run: () => void locationPicker.openIds([...selection.selected])
  });

  readonly paste: BulkMenuAction | null = $derived.by(() => {
    const copied = this.#copied;
    if (!copied) return null;
    return {
      label: 'Paste location',
      icon: mdiContentPaste,
      bind: 'pasteLocation',
      detail: locationName(copied),
      disabled: this.#blocked,
      run: () => void pasteLocationTo([...selection.selected])
    };
  });

  readonly saved: BulkMenuAction[] = $derived(
    places.saved.slice(0, MENU_SAVED).map((place) => ({
      label: place.name,
      icon: mdiStar,
      bind: null,
      detail: spotDetail(place, place.name),
      disabled: this.#blocked,
      run: () => void placeSelection(place)
    }))
  );

  readonly recent: BulkMenuAction[] = $derived(
    places.freshRecent.slice(0, MENU_RECENT).map((spot) => {
      const label = locationName(locationOfSpot(spot));
      return {
        label,
        icon: mdiHistory,
        bind: null,
        detail: spotDetail(spot, label),
        disabled: this.#blocked,
        run: () => void placeSelection(spot)
      };
    })
  );

  readonly track: BulkMenuAction = $derived({
    label: 'Match to a track…',
    icon: mdiMapMarkerPath,
    bind: null,
    detail: null,
    disabled: this.#blocked,
    run: () => void matchTrackFor([...selection.selected])
  });

  readonly copy: BulkMenuAction | null = $derived.by(() => {
    const single = bulkScope.single;
    if (!single || !locationOf(single.exifInfo)) return null;
    return {
      label: 'Copy location',
      icon: mdiContentCopy,
      bind: 'copyLocation',
      detail: null,
      disabled: bulkActions.metaBusy,
      run: () => copyLocationOf(single)
    };
  });
}

export const bulkLocation = new BulkLocation();
