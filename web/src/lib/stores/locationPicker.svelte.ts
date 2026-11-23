import {
  applyLocation,
  locationOf,
  resolveAssets,
  type GeoLocation
} from '#lib/browse/locationOps.ts';
import { locationClipboard } from '#lib/stores/locationClipboard.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';

export interface LocationRequest {
  assets: AssetSummary[];
  start: GeoLocation | null;
}

class LocationPickerStore {
  request = $state<LocationRequest | null>(null);
  saving = $state(false);

  open = (assets: readonly AssetSummary[]): void => {
    if (assets.length === 0) return;
    const start =
      assets.map((asset) => locationOf(asset.exifInfo)).find((at) => at !== null) ??
      locationClipboard.copied;
    this.request = { assets: [...assets], start };
  };

  openIds = async (ids: readonly string[]): Promise<void> => {
    try {
      this.open(await resolveAssets(ids));
    } catch (error) {
      toasts.fail('Set location', error);
    }
  };

  close = (): void => {
    if (!this.saving) this.request = null;
  };

  save = async (to: GeoLocation): Promise<void> => {
    const request = this.request;
    if (!request || this.saving) return;
    this.saving = true;
    try {
      if (await applyLocation(request.assets, to)) this.request = null;
    } finally {
      this.saving = false;
    }
  };
}

export const locationPicker = new LocationPickerStore();
