import { readStored, writeStored } from '$lib/utils/storage';
import { restoreExportForm, type Destination, type ExportForm } from './settings';

const STORAGE_KEY = 'immich-edit:exportSettings';

interface PersistedExportSettings {
  form: Partial<Record<keyof ExportForm, unknown>>;
  destination: Destination;
}

class ExportSettingsStore {
  form = $state<ExportForm>(restoreExportForm(undefined));
  destination = $state<Destination>('download');

  constructor() {
    const stored = readStored<PersistedExportSettings>(STORAGE_KEY);
    this.form = restoreExportForm(stored?.form);
    if (stored?.destination === 'immich') this.destination = 'immich';
    $effect.root(() => {
      $effect(() => {
        writeStored(STORAGE_KEY, {
          form: $state.snapshot(this.form),
          destination: this.destination
        } satisfies PersistedExportSettings);
      });
    });
  }
}

export const exportSettings = new ExportSettingsStore();
