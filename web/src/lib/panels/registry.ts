import type { Component } from 'svelte';
import HistogramPanel from './Histogram.svelte';
import WhiteBalancePanel from './WhiteBalance.svelte';
import BasicPanel from './Basic.svelte';
import CurvesPanel from './Curves.svelte';
import HslPanel from './Hsl.svelte';
import ColorGradingPanel from './ColorGrading.svelte';
import InfoPanel from './Info.svelte';
import TagsPanel from './Tags.svelte';

export interface PanelDef {
  id: string;
  title: string;
  component: Component;
  defaultOpen: boolean;
}

export const developPanels: PanelDef[] = [
  { id: 'histogram', title: 'Histogram', component: HistogramPanel, defaultOpen: true },
  { id: 'wb', title: 'White Balance', component: WhiteBalancePanel, defaultOpen: true },
  { id: 'basic', title: 'Tone', component: BasicPanel, defaultOpen: true },
  { id: 'curves', title: 'Curves', component: CurvesPanel, defaultOpen: true },
  { id: 'hsl', title: 'HSL', component: HslPanel, defaultOpen: true },
  { id: 'color-grading', title: 'Color Grading', component: ColorGradingPanel, defaultOpen: true },
  { id: 'info', title: 'Info', component: InfoPanel, defaultOpen: false },
  { id: 'tags', title: 'Tags', component: TagsPanel, defaultOpen: false },
];
