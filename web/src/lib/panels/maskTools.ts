import { goto } from '$app/navigation';
import type { MaskKind } from '$lib/api/masks';
import { keyLabel } from '$lib/keybinds';
import { editor } from '$lib/stores/editor.svelte';
import { maskModels } from '$lib/stores/maskModels.svelte';
import { session } from '$lib/stores/session.svelte';
import { toasts } from '$lib/stores/toasts.svelte';
import type { MaskComponentKind, MaskComponentMode } from '$lib/types/edits';
import { generatedLabel, manualKind, type ManualTool } from '$lib/types/masks';
import {
  mdiBrightness6,
  mdiBrush,
  mdiCircleOutline,
  mdiGradientHorizontal,
  mdiMinus,
  mdiPalette,
  mdiPlus,
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
    toasts.push(
      'info',
      `Click the photo to build a mask. ${keyLabel('Shift')}-click excludes an area, and clicking a dot deletes it.`
    );
    return;
  }
  if (target.layerId)
    await editor.addGeneratedComponent(target.layerId, kind, target.mode, maskClass);
  else await editor.addGeneratedLayer(kind, maskClass);
}

export function armBoxTool(target: MaskTarget): void {
  armClickTool(target, true);
  toasts.push('info', 'Drag a box around the subject.');
}

export async function addBackground(target: MaskTarget): Promise<void> {
  if (target.layerId)
    await editor.addGeneratedComponent(target.layerId, 'subject', target.mode, undefined, true);
  else await editor.addGeneratedLayer('subject', undefined, true);
}

export function refineWith(layerId: string, negative: boolean): void {
  editor.clickTool = { active: true, negative, box: false, layerId, mode: 'add' };
}

export function stopClickTool(): void {
  editor.clickTool = { active: false, negative: false, box: false, layerId: null, mode: 'add' };
}

function armClickTool(target: MaskTarget, box: boolean): void {
  if (!maskModels.clickInstalled) {
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
