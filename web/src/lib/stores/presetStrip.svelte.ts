import type { ApplyPresetOptions } from '#lib/api/jobs.ts';
import type { Preset } from '#lib/api/presets.ts';
import type { PillText } from '#lib/editor/pill.ts';
import { defaultApplyOptions } from '#lib/edits/applyPreset.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { presets } from '#lib/stores/presets.svelte.ts';

export interface GroupOption {
  value: string | null;
  label: string;
}

interface Previewing {
  name: string;
  held: boolean;
}

class PresetStripStore {
  applyOptions = $state<ApplyPresetOptions>(defaultApplyOptions());
  private picked = $state<string | null>(null);
  private previewing = $state<Previewing | null>(null);

  readonly groups = $derived(
    presets.grouped
      .map((g) => g.group)
      .sort((a, b) => Number(a === '') - Number(b === '') || a.localeCompare(b))
  );
  readonly groupOptions = $derived<GroupOption[]>([
    { value: null, label: 'All' },
    ...this.groups.map((g) => ({ value: g, label: g || 'Ungrouped' }))
  ]);
  readonly group = $derived(
    this.picked !== null && this.groups.includes(this.picked) ? this.picked : null
  );
  readonly shown = $derived(
    this.group === null
      ? presets.presets
      : presets.presets.filter((p) => (p.group_name ?? '') === this.group)
  );

  get pill(): PillText | null {
    const previewing = this.previewing;
    if (!previewing || editor.editsPreview?.owner !== 'preset') return null;
    return {
      lead: 'Previewing',
      label: previewing.name,
      tail: previewing.held ? '· release to cancel' : '',
      mono: false
    };
  }

  ensureLoaded = (): void => {
    if (!presets.loaded && !presets.loading) void presets.load();
  };

  pick = (group: string | null): void => {
    this.picked = group;
  };

  preview = (preset: Preset, held = false): void => {
    this.previewing = { name: preset.name, held };
    editor.previewPreset({ manifest: preset.manifest, opts: $state.snapshot(this.applyOptions) });
  };

  endPreview = (): void => {
    this.previewing = null;
    editor.endPresetPreview();
  };

  apply = (preset: Preset): void => {
    this.previewing = null;
    void editor.applyPreset(preset.manifest, $state.snapshot(this.applyOptions), preset.name);
  };
}

export const presetStrip = new PresetStripStore();
