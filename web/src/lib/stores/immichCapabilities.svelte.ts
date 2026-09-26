import { getImmichCapabilities } from '$lib/api/immich';

class ImmichCapabilitiesStore {
  version = $state<string | null>(null);
  minRatingFilter = $state(false);

  load = async (): Promise<void> => {
    try {
      const caps = await getImmichCapabilities();
      this.version = caps.immich_version;
      this.minRatingFilter = caps.min_rating_filter;
    } catch {
      this.version = null;
      this.minRatingFilter = false;
    }
  };
}

export const immichCapabilities = new ImmichCapabilitiesStore();
