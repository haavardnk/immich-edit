const STORAGE_KEY = 'immich-edit:settings';

type Persisted = {
  metadataPushConsented: boolean;
};

function loadPersisted(): Persisted {
  const fallback: Persisted = { metadataPushConsented: false };
  if (typeof localStorage === 'undefined') return fallback;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return fallback;
    const parsed = JSON.parse(raw) as Partial<Persisted>;
    return {
      metadataPushConsented:
        typeof parsed.metadataPushConsented === 'boolean'
          ? parsed.metadataPushConsented
          : fallback.metadataPushConsented
    };
  } catch {
    return fallback;
  }
}

class SettingsStore {
  metadataPushConsented = $state(false);

  constructor() {
    this.metadataPushConsented = loadPersisted().metadataPushConsented;
  }

  private persist(): void {
    if (typeof localStorage === 'undefined') return;
    const data: Persisted = { metadataPushConsented: this.metadataPushConsented };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
  }

  setMetadataPushConsented(on: boolean): void {
    this.metadataPushConsented = on;
    this.persist();
  }
}

export const settings = new SettingsStore();
