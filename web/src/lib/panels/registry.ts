import { DESKTOP_PANELS } from '#lib/editor/params/catalog.ts';
import type { DesktopPanelId } from '#lib/editor/params/types.ts';
import type { Component } from 'svelte';
import BasicPanel from './Basic.svelte';
import CurvesPanel from './Curves.svelte';
import HslPanel from './Hsl.svelte';
import ColorGradingPanel from './ColorGrading.svelte';
import BwPanel from './Bw.svelte';
import LutPanel from './Lut.svelte';
import DcpPanel from './Dcp.svelte';
import DetailPanel from './Detail.svelte';
import LensPanel from './Lens.svelte';
import LensBlurPanel from './LensBlur.svelte';
import EffectsPanel from './Effects.svelte';

export interface PanelDef {
  id: DesktopPanelId;
  title: string;
  component: Component;
  defaultOpen: boolean;
}

export const SCOPES_PANEL = 'scopes';

const COMPONENTS: Record<DesktopPanelId, Component> = {
  dcp: DcpPanel,
  basic: BasicPanel,
  curves: CurvesPanel,
  hsl: HslPanel,
  'color-grading': ColorGradingPanel,
  bw: BwPanel,
  lut: LutPanel,
  detail: DetailPanel,
  lens: LensPanel,
  'lens-blur': LensBlurPanel,
  effects: EffectsPanel
};

const DEFAULT_OPEN: readonly DesktopPanelId[] = ['basic'];

export const developPanels: PanelDef[] = DESKTOP_PANELS.map(({ id, title }) => ({
  id,
  title,
  component: COMPONENTS[id],
  defaultOpen: DEFAULT_OPEN.includes(id)
}));

const defaultOpenPanels = [
  SCOPES_PANEL,
  ...developPanels.filter((panel) => panel.defaultOpen).map((panel) => panel.id)
];

export function openDevelopPanels(stored: readonly string[] | null): Set<string> {
  return new Set(stored ?? defaultOpenPanels);
}
