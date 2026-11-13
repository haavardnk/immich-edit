import type { MaskKind } from '#lib/api/masks.ts';
import type { KeybindId } from '#lib/shortcuts/table.ts';
import { generatedLabel, type ManualTool } from '#lib/types/masks.ts';
import {
  mdiAccountMultipleOutline,
  mdiAccountOutline,
  mdiBrightness6,
  mdiBrush,
  mdiCircleOutline,
  mdiCursorDefaultClick,
  mdiGradientHorizontal,
  mdiLayersOutline,
  mdiPalette,
  mdiSelectDrag,
  mdiShapeOutline,
  mdiVectorDifference,
  mdiVectorPolygon,
  mdiWeatherCloudy
} from '@mdi/js';

export type ToolPick =
  | { kind: 'manual'; tool: ManualTool }
  | { kind: 'ai'; mask: MaskKind; installed: boolean; maskClass?: string }
  | { kind: 'background' }
  | { kind: 'box' };

export type AiTool = {
  id: string;
  label: string;
  hint: string;
  icon: string;
  installed: boolean;
  scene: boolean;
  generates: boolean;
  pick: ToolPick;
};

export type ManualToolEntry = {
  tool: ManualTool;
  label: string;
  hint: string;
  icon: string;
  bind?: KeybindId;
};

export const MANUAL_TOOLS: readonly ManualToolEntry[] = [
  {
    tool: 'linear',
    label: 'Linear gradient',
    hint: 'Fades across a straight edge',
    icon: mdiGradientHorizontal,
    bind: 'addLinearLayer'
  },
  {
    tool: 'radial',
    label: 'Radial gradient',
    hint: 'Fades out from an ellipse',
    icon: mdiCircleOutline,
    bind: 'addRadialLayer'
  },
  {
    tool: 'brush',
    label: 'Brush',
    hint: 'Paint the area by hand',
    icon: mdiBrush,
    bind: 'addBrushLayer'
  },
  {
    tool: 'polygon',
    label: 'Polygon',
    hint: 'Drag the corners to fit a shape',
    icon: mdiVectorPolygon
  },
  {
    tool: 'luma_range',
    label: 'Luminance range',
    hint: 'Picks a brightness band',
    icon: mdiBrightness6
  },
  {
    tool: 'color_range',
    label: 'Color range',
    hint: 'Picks colors near a sample',
    icon: mdiPalette
  }
];

const AI_ORDER: readonly MaskKind[] = ['subject', 'people', 'sky', 'depth', 'semantic', 'click'];

const AI_LOOKS: Record<string, { hint: string; icon: string }> = {
  subject: { hint: 'Isolates the main subject', icon: mdiAccountOutline },
  people: { hint: 'Finds every person', icon: mdiAccountMultipleOutline },
  sky: { hint: 'Finds the sky', icon: mdiWeatherCloudy },
  depth: { hint: 'Selects a distance band', icon: mdiLayersOutline },
  semantic: { hint: 'Water, foliage, buildings and more', icon: mdiShapeOutline },
  click: { hint: 'Click the photo to select', icon: mdiCursorDefaultClick }
};

export const MISSING_MODEL = 'Needs a model download';

function rank(kind: MaskKind): number {
  const at = AI_ORDER.indexOf(kind);
  return at < 0 ? AI_ORDER.length : at;
}

export function aiTools(
  kinds: readonly { kind: MaskKind; installed: boolean }[],
  touch = false
): AiTool[] {
  const sorted = [...kinds].sort((a, b) => rank(a.kind) - rank(b.kind));
  return sorted.flatMap(({ kind, installed }) => {
    const look = AI_LOOKS[kind] ?? { hint: '', icon: mdiShapeOutline };
    const hint = kind === 'click' && touch ? 'Tap the photo to select' : look.hint;
    const entry: AiTool = {
      id: kind,
      label: generatedLabel(kind),
      hint: installed ? hint : MISSING_MODEL,
      icon: look.icon,
      installed,
      scene: kind === 'semantic' && installed,
      generates: kind !== 'click',
      pick: { kind: 'ai', mask: kind, installed }
    };
    if (!installed) return [entry];
    if (kind === 'subject') {
      return [
        entry,
        {
          id: 'background',
          label: 'Background',
          hint: 'Everything but the subject',
          icon: mdiVectorDifference,
          installed,
          scene: false,
          generates: true,
          pick: { kind: 'background' }
        }
      ];
    }
    if (kind === 'click') {
      return [
        entry,
        {
          id: 'box',
          label: 'Box select',
          hint: 'Drag a box around a subject',
          icon: mdiSelectDrag,
          installed,
          scene: false,
          generates: false,
          pick: { kind: 'box' }
        }
      ];
    }
    return [entry];
  });
}
