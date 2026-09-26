import {
  createExportPreset,
  deleteExportPreset,
  listExportPresets,
  updateExportPreset,
  type ExportPreset
} from '$lib/api/exportPresets';
import { toasts } from '$lib/stores/toasts.svelte';
import { exportSettings } from './exportSettings.svelte';
import { pruneLibraryIds, restoreExportForm } from './settings';

class ExportPresetsStore {
  presets = $state<ExportPreset[]>([]);
  loaded = $state(false);

  load = async (): Promise<void> => {
    try {
      this.presets = await listExportPresets();
      this.loaded = true;
    } catch (e) {
      toasts.fail('Failed to load export presets', e, 8000);
    }
  };

  save = async (name: string): Promise<void> => {
    const form = $state.snapshot(exportSettings.form) as unknown as Record<string, unknown>;
    const existing = this.presets.find((p) => p.name.toLowerCase() === name.toLowerCase());
    try {
      const saved = existing
        ? await updateExportPreset(existing.id, { name: existing.name, form })
        : await createExportPreset({ name, form });
      this.presets = [...this.presets.filter((p) => p.id !== saved.id), saved].sort((a, b) =>
        a.name.localeCompare(b.name, undefined, { sensitivity: 'base' })
      );
      toasts.push('success', `Saved export preset "${saved.name}"`, 5000);
    } catch (e) {
      toasts.fail('Failed to save export preset', e, 8000);
    }
  };

  apply = (preset: ExportPreset): void => {
    Object.assign(exportSettings.form, restoreExportForm(preset.form));
    pruneLibraryIds(exportSettings.form);
  };

  remove = async (preset: ExportPreset): Promise<void> => {
    try {
      await deleteExportPreset(preset.id);
      this.presets = this.presets.filter((p) => p.id !== preset.id);
    } catch (e) {
      toasts.fail('Failed to delete export preset', e, 8000);
    }
  };
}

export const exportPresets = new ExportPresetsStore();
