import { readStored, writeStored } from '$lib/utils/storage';

const STORAGE_KEY = 'immich-edit:settings';

type Persisted = {
  metadataPushConsented: boolean;
};

class SettingsStore {
  metadataPushConsented = $state(false);

  constructor() {
    const stored = readStored<Persisted>(STORAGE_KEY);
    if (typeof stored?.metadataPushConsented === 'boolean')
      this.metadataPushConsented = stored.metadataPushConsented;
  }

  setMetadataPushConsented(on: boolean): void {
    this.metadataPushConsented = on;
    writeStored(STORAGE_KEY, { metadataPushConsented: on } satisfies Persisted);
  }
}

export const settings = new SettingsStore();
