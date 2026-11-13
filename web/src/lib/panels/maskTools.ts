import { goto } from '$app/navigation';
import type { MaskKind } from '#lib/api/masks.ts';
import { keyLabel } from '#lib/shortcuts/chord.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { maskModels } from '#lib/stores/maskModels.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { MaskComponentKind, MaskComponentMode } from '#lib/types/edits.ts';
import { generatedLabel, manualKind, type ManualTool } from '#lib/types/masks.ts';
import {
  mdiBrightness6,
  mdiBrush,
  mdiCircleOutline,
  mdiGradientHorizontal,
  mdiMinus,
  mdiMinusCircleOutline,
  mdiPalette,
  mdiPlus,
  mdiPlusCircleOutline,
  mdiSetCenter,
  mdiVectorPolygon
} from '@mdi/js';

export interface MaskTarget {
  layerId: string | null;
  mode: MaskComponentMode;
}

const KIND_ICONS: Record<MaskComponentKind['kind'], string> = {
  linear: mdiGradientHorizontal,
  radial: mdiCircleOutline,
  brush: mdiBrush,
  polygon: mdiVectorPolygon,
  luma_range: mdiBrightness6,
  color_range: mdiPalette
};

export const MODES: { value: MaskComponentMode; icon: string; hint: string }[] = [
  { value: 'add', icon: mdiPlus, hint: 'Add: add this shape to the mask' },
  { value: 'subtract', icon: mdiMinus, hint: 'Subtract: cut this shape out of the mask' },
  { value: 'intersect', icon: mdiSetCenter, hint: 'Intersect: keep only where this shape overlaps' }
];

const MODE_BADGES: Record<MaskComponentMode, { icon: string; tone: string }> = {
  add: { icon: mdiPlusCircleOutline, tone: 'text-success' },
  subtract: { icon: mdiMinusCircleOutline, tone: 'text-danger' },
  intersect: { icon: mdiSetCenter, tone: 'text-primary' }
};

export function modeBadge(mode: MaskComponentMode): { icon: string; tone: string; hint: string } {
  const hint = MODES.find((entry) => entry.value === mode)?.hint ?? '';
  return { ...MODE_BADGES[mode], hint };
}

export function kindIcon(kind: MaskComponentKind): string {
  return KIND_ICONS[kind.kind];
}

export function modeVerb(mode: MaskComponentMode): string {
  if (mode === 'subtract') return 'Subtract';
  if (mode === 'intersect') return 'Intersect';
  return 'Add';
}

export function promptInstall(kind: MaskKind): void {
  if (session.isAdmin) {
    void goto('/settings');
    return;
  }
  toasts.push(
    'info',
    `${generatedLabel(kind)} masks need a model. Ask an administrator to download one in Settings.`
  );
}

export async function addManualTool(target: MaskTarget, tool: ManualTool): Promise<void> {
  if (tool === 'polygon') {
    editor.beginPolygon(target.layerId, target.mode);
    toasts.push(
      'info',
      `Click to place corners. Click the first one, or press ${keyLabel('Enter')}, to close.`
    );
    return;
  }
  const kind = manualKind(tool);
  if (!kind) {
    if (target.layerId) await editor.addBrushComponent(target.layerId, target.mode);
    else await editor.addBrushLayer();
    return;
  }
  if (target.layerId) await editor.addMaskComponent(target.layerId, kind, target.mode);
  else await editor.addMaskLayer(kind);
}

export async function addAiTool(
  target: MaskTarget,
  kind: MaskKind,
  installed: boolean,
  maskClass?: string
): Promise<void> {
  if (!installed) {
    promptInstall(kind);
    return;
  }
  if (kind === 'click') {
    armClickTool(target, false);
    return;
  }
  if (target.layerId)
    await editor.addGeneratedComponent(target.layerId, kind, target.mode, maskClass);
  else await editor.addGeneratedLayer(kind, maskClass);
}

export function armBoxTool(target: MaskTarget): void {
  armClickTool(target, true);
}

export async function addBackground(target: MaskTarget): Promise<void> {
  if (target.layerId)
    await editor.addGeneratedComponent(target.layerId, 'subject', target.mode, undefined, true);
  else await editor.addGeneratedLayer('subject', undefined, true);
}

export function refineWith(layerId: string | null, negative: boolean): void {
  const tool = editor.clickTool;
  if (tool.active && !tool.box && tool.layerId === layerId) {
    tool.negative = negative;
    return;
  }
  editor.clickTool = { active: true, negative, box: false, layerId, mode: 'add' };
}

function armClickTool(target: MaskTarget, box: boolean): void {
  if (!maskModels.installed('click')) {
    promptInstall('click');
    return;
  }
  editor.setActiveMaskComponent(null);
  editor.clickTool = {
    active: true,
    negative: false,
    box,
    layerId: target.layerId,
    mode: target.mode
  };
}
