import { getImmichCapabilities } from '$lib/api/immich';

class ImmichCapabilitiesStore {
  structuredSearch = $state(false);
  smartSearch = $state(true);
  ocr = $state(true);

  load = async (): Promise<void> => {
    try {
      const capabilities = await getImmichCapabilities();
      this.structuredSearch = capabilities.structured_search;
      this.smartSearch = capabilities.smart_search;
      this.ocr = capabilities.ocr;
    } catch {
      this.structuredSearch = false;
    }
  };
}

export const immichCapabilities = new ImmichCapabilitiesStore();
