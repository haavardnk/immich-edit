import type { ExportPreset } from '#lib/api/exportPresets.ts';
import { MAX_JOB_PRESETS } from '#lib/api/jobs.ts';
import { countOf } from '#lib/utils/plural.ts';
import { SvelteSet } from 'svelte/reactivity';
import { exportPresets } from './exportPresets.svelte';
import type { Destination } from './settings';

export class PresetSelection {
  private ids = new SvelteSet<string>();

  readonly picked = $derived<ExportPreset[]>(
    exportPresets.presets.filter((preset) => this.ids.has(preset.id))
  );

  readonly jobCount = $derived(new Set(this.picked.map((preset) => preset.destination)).size);

  readonly tooMany = $derived(
    (['download', 'immich'] as const).some(
      (destination) =>
        this.picked.filter((preset) => preset.destination === destination).length > MAX_JOB_PRESETS
    )
  );

  readonly note = $derived.by(() => {
    if (this.tooMany) return `Up to ${MAX_JOB_PRESETS} presets per destination`;
    if (this.picked.length < 2) return '';
    const jobs = this.jobCount === 1 ? 'one job' : countOf(this.jobCount, 'job');
    return `${this.picked.length} selected · runs as ${jobs}`;
  });

  has = (id: string): boolean => this.ids.has(id);

  toggle = (id: string, on: boolean): void => {
    if (on) this.ids.add(id);
    else this.ids.delete(id);
  };

  drop = (destination: Destination): void => {
    for (const preset of this.picked) {
      if (preset.destination === destination) this.ids.delete(preset.id);
    }
  };

  clear = (): void => this.ids.clear();
}
