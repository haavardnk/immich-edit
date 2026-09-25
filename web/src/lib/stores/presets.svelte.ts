import {
  createPreset,
  deletePreset,
  listPresets,
  updatePreset,
  type Preset,
  type PresetInput
} from '$lib/api/presets';
import {
  PRESET_LIBRARY_FILE,
  importSummary,
  newPresets,
  parsePresetFile,
  presetFileJson,
  presetFileName
} from '$lib/presetFile';
import { toasts } from '$lib/stores/toasts.svelte';
import { downloadBlob } from '$lib/utils/download';
import { errorMessage } from '$lib/utils/errors';

class PresetsStore {
  presets = $state<Preset[]>([]);
  loading = $state(false);
  loaded = $state(false);

  grouped = $derived.by(() => {
    const map = new Map<string, Preset[]>();
    for (const p of this.presets) {
      const key = p.group_name ?? '';
      const list = map.get(key);
      if (list) {
        list.push(p);
      } else {
        map.set(key, [p]);
      }
    }
    return [...map.entries()].map(([group, items]) => ({ group, items }));
  });

  load = async (): Promise<void> => {
    if (this.loading) return;
    this.loading = true;
    try {
      this.presets = await listPresets();
      this.loaded = true;
    } catch (e) {
      toasts.fail('Failed to load presets', e, 8000);
    } finally {
      this.loading = false;
    }
  };

  create = async (input: PresetInput): Promise<Preset | null> => {
    try {
      const created = await createPreset(input);
      this.presets = [...this.presets, created];
      toasts.push('success', `Saved preset "${created.name}"`, 5000);
      return created;
    } catch (e) {
      toasts.fail('Failed to save preset', e, 8000);
      return null;
    }
  };

  update = async (id: string, input: PresetInput): Promise<void> => {
    try {
      const updated = await updatePreset(id, input);
      this.presets = this.presets.map((p) => (p.id === id ? updated : p));
    } catch (e) {
      toasts.fail('Failed to update preset', e, 8000);
    }
  };

  remove = async (id: string): Promise<void> => {
    try {
      await deletePreset(id);
      this.presets = this.presets.filter((p) => p.id !== id);
    } catch (e) {
      toasts.fail('Failed to delete preset', e, 8000);
    }
  };

  exportAll = (): void => {
    if (this.presets.length === 0) return;
    this.download(this.presets, PRESET_LIBRARY_FILE);
  };

  exportOne = (preset: Preset): void => {
    this.download([preset], presetFileName(preset));
  };

  private download(list: Preset[], fileName: string): void {
    downloadBlob(new Blob([presetFileJson(list)], { type: 'application/json' }), fileName);
  }

  importFile = async (file: File): Promise<void> => {
    let incoming: PresetInput[];
    try {
      incoming = parsePresetFile(await file.text());
    } catch (e) {
      toasts.fail('Failed to import presets', e, 8000);
      return;
    }
    if (!this.loaded) await this.load();
    const { fresh, skipped } = newPresets(incoming, this.presets);
    const created: Preset[] = [];
    let firstError: unknown = null;
    for (const input of fresh) {
      try {
        created.push(await createPreset(input));
      } catch (e) {
        firstError ??= e;
      }
    }
    this.presets = [...this.presets, ...created];
    const failed = fresh.length - created.length;
    const summary = importSummary(created.length, skipped, failed);
    if (failed > 0) {
      toasts.push('error', `${summary}: ${errorMessage(firstError)}`, 8000);
    } else {
      toasts.push(created.length > 0 ? 'success' : 'info', summary, 8000);
    }
  };
}

export const presets = new PresetsStore();
