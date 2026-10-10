import { getImmichCapabilities } from '$lib/api/immich';

class ImmichCapabilitiesStore {
  structuredSearch = $state(false);

  load = async (): Promise<void> => {
    try {
      this.structuredSearch = (await getImmichCapabilities()).structured_search;
    } catch {
      this.structuredSearch = false;
    }
  };
}

export const immichCapabilities = new ImmichCapabilitiesStore();
