import { getImmichCapabilities } from '#lib/api/immich.ts';

class ImmichCapabilitiesStore {
  structuredSearch = $state(false);
  smartSearch = $state(true);
  ocr = $state(true);
  trash = $state(false);
  trashDays = $state<number | null>(null);
  externalHost = $state<string | null>(null);
  version = $state<string | null>(null);

  load = async (): Promise<void> => {
    try {
      const capabilities = await getImmichCapabilities();
      this.structuredSearch = capabilities.structured_search;
      this.smartSearch = capabilities.smart_search;
      this.ocr = capabilities.ocr;
      this.trash = capabilities.trash;
      this.trashDays = capabilities.trash_days;
      this.externalHost = capabilities.external_host;
      this.version = capabilities.immich_version;
    } catch {
      this.structuredSearch = false;
    }
  };
}

export const immichCapabilities = new ImmichCapabilitiesStore();
