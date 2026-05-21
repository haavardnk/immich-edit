import type { Component } from 'svelte';
import HistogramPanel from './Histogram.svelte';
import BasicPanel from './Basic.svelte';
import ColorPanel from './Color.svelte';
import TransformPanel from './Transform.svelte';
import ExportPanel from './Export.svelte';

export interface PanelDef {
  id: string;
  title: string;
  component: Component;
  defaultOpen: boolean;
}

export const panels: PanelDef[] = [
  { id: 'histogram', title: 'Histogram', component: HistogramPanel, defaultOpen: true },
  { id: 'basic', title: 'Light', component: BasicPanel, defaultOpen: true },
  { id: 'color', title: 'Color', component: ColorPanel, defaultOpen: true },
  { id: 'transform', title: 'Geometry', component: TransformPanel, defaultOpen: false },
  { id: 'export', title: 'Export', component: ExportPanel, defaultOpen: true }
];
