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
  icon: string;
  installed: boolean;
  scene: boolean;
  generates: boolean;
  pick: ToolPick;
};

export type ManualToolEntry = {
  tool: ManualTool;
  label: string;
  icon: string;
  bind?: KeybindId;
};

export const MANUAL_TOOLS: readonly ManualToolEntry[] = [
  {
    tool: 'linear',
    label: 'Linear gradient',
    icon: mdiGradientHorizontal,
    bind: 'addLinearLayer'
  },
  {
    tool: 'radial',
    label: 'Radial gradient',
    icon: mdiCircleOutline,
    bind: 'addRadialLayer'
  },
  {
    tool: 'brush',
    label: 'Brush',
    icon: mdiBrush,
    bind: 'addBrushLayer'
  },
  {
    tool: 'polygon',
    label: 'Polygon',
    icon: mdiVectorPolygon
  },
  {
    tool: 'luma_range',
    label: 'Luminance range',
    icon: mdiBrightness6
  },
  {
    tool: 'color_range',
    label: 'Color range',
    icon: mdiPalette
  }
];

const AI_ORDER: readonly MaskKind[] = ['subject', 'people', 'sky', 'depth', 'semantic', 'click'];

const AI_ICONS: Record<string, string> = {
  subject: mdiAccountOutline,
  people: mdiAccountMultipleOutline,
  sky: mdiWeatherCloudy,
  depth: mdiLayersOutline,
  semantic: mdiShapeOutline,
  click: mdiCursorDefaultClick
};

export const MISSING_MODEL = 'Needs a model download';

function rank(kind: MaskKind): number {
  const at = AI_ORDER.indexOf(kind);
  return at < 0 ? AI_ORDER.length : at;
}

export function aiTools(kinds: readonly { kind: MaskKind; installed: boolean }[]): AiTool[] {
  const sorted = [...kinds].sort((a, b) => rank(a.kind) - rank(b.kind));
  return sorted.flatMap(({ kind, installed }) => {
    const entry: AiTool = {
      id: kind,
      label: generatedLabel(kind),
      icon: AI_ICONS[kind] ?? mdiShapeOutline,
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
