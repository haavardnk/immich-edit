import type { EarlierExports, EarlierQuery } from './earlierExports.svelte';
import type { PresetSelection } from './presetSelection.svelte';
import type { ExportScope } from './scope';

export interface PresetExporter {
  readonly presets: PresetSelection;
  readonly exportingPresets: boolean;
  readonly earlierPresets: EarlierExports;
  readonly presetQuery: EarlierQuery | null;
  readonly scope: ExportScope;
  readonly exportPresets: () => Promise<boolean>;
}
