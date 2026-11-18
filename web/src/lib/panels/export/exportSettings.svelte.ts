import {
  restoreExportForm,
  type Destination,
  type ExportFormValues
} from '#lib/panels/export/settings.ts';
import { readStored, writeStored } from '#lib/utils/storage.ts';

const STORAGE_KEY = 'immich-edit:exportSettings';

interface PersistedExportSettings {
  form: Partial<Record<keyof ExportFormValues, unknown>>;
  destination: Destination;
}

class ExportSettingsStore {
  form = $state<ExportFormValues>(restoreExportForm(undefined));
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
