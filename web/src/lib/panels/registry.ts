import type { Component } from 'svelte';
import HistogramPanel from './Histogram.svelte';
import BasicPanel from './Basic.svelte';
import CurvesPanel from './Curves.svelte';
import HslPanel from './Hsl.svelte';
import ColorGradingPanel from './ColorGrading.svelte';
import LutPanel from './Lut.svelte';
import DcpPanel from './Dcp.svelte';
import DetailPanel from './Detail.svelte';
import LensPanel from './Lens.svelte';
import EffectsPanel from './Effects.svelte';
import PresetsPanel from './Presets.svelte';
import VersionsPanel from './Versions.svelte';

export interface PanelDef {
  id: string;
  title: string;
  component: Component;
  defaultOpen: boolean;
}

export const developPanels: PanelDef[] = [
  { id: 'histogram', title: 'Histogram', component: HistogramPanel, defaultOpen: true },
  { id: 'dcp', title: 'Camera Profile', component: DcpPanel, defaultOpen: false },
  { id: 'presets', title: 'Presets', component: PresetsPanel, defaultOpen: false },
  { id: 'basic', title: 'Basic', component: BasicPanel, defaultOpen: true },
  { id: 'curves', title: 'Curves', component: CurvesPanel, defaultOpen: false },
  { id: 'hsl', title: 'HSL', component: HslPanel, defaultOpen: false },
  { id: 'color-grading', title: 'Color Grading', component: ColorGradingPanel, defaultOpen: false },
  { id: 'lut', title: 'LUT', component: LutPanel, defaultOpen: false },
  { id: 'detail', title: 'Detail', component: DetailPanel, defaultOpen: false },
  { id: 'lens', title: 'Lens Corrections', component: LensPanel, defaultOpen: false },
  { id: 'effects', title: 'Effects', component: EffectsPanel, defaultOpen: false },
  { id: 'versions', title: 'Versions', component: VersionsPanel, defaultOpen: false }
];
