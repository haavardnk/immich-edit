import type { Component } from 'svelte';
import HistogramPanel from './Histogram.svelte';
import WhiteBalancePanel from './WhiteBalance.svelte';
import BasicPanel from './Basic.svelte';
import ColorPanel from './Color.svelte';

export interface PanelDef {
  id: string;
  title: string;
  component: Component;
  defaultOpen: boolean;
}

export const developPanels: PanelDef[] = [
  { id: 'histogram', title: 'Histogram', component: HistogramPanel, defaultOpen: true },
  { id: 'wb', title: 'White Balance', component: WhiteBalancePanel, defaultOpen: false },
  { id: 'basic', title: 'Tone', component: BasicPanel, defaultOpen: true },
  { id: 'color', title: 'Color', component: ColorPanel, defaultOpen: false },
];
