import {
  N_MAX_COMPONENTS_PER_LAYER,
  N_MAX_MASK_LAYERS,
  N_MAX_TOTAL_COMPONENTS,
  type Edits,
  type GeneratedMeta,
  type MaskComponent,
  type MaskComponentKind,
  type MaskComponentMode,
  type MaskLayer,
  type MaskedEditKey,
  type MaskedEdits,
  type Vec2f
} from './edits';
import type { MaskKind } from '$lib/api/masks';
import { v4 as uuidv4 } from 'uuid';

export type ManualTool = 'linear' | 'radial' | 'brush' | 'polygon' | 'luma_range' | 'color_range';

const PALETTE = [
  '#ff3b30',
  '#ff9500',
  '#ffcc00',
  '#34c759',
  '#5ac8fa',
  '#007aff',
  '#af52de',
  '#ff2d55'
];

function nextId(): string {
  return uuidv4();
}

export function defaultMaskColor(index: number): string {
  return PALETTE[index % PALETTE.length];
}

export function defaultLinear(): MaskComponentKind {
  return {
    kind: 'linear',
    p0: { x: 0.5, y: 0.1 },
    p1: { x: 0.5, y: 0.9 },
    feather: 0.5
  };
}

export function defaultRadial(): MaskComponentKind {
  return {
    kind: 'radial',
    center: { x: 0.5, y: 0.5 },
    radius_xy: { x: 0.25, y: 0.25 },
    feather: 0.5
  };
}

export function defaultBrush(rasterId: string): MaskComponentKind {
  return { kind: 'brush', raster_id: rasterId };
}

export function defaultLumaRange(): MaskComponentKind {
  return { kind: 'luma_range', min: 0.25, max: 0.75, softness: 0.1 };
}

export const MAX_POLYGON_POINTS = 64;

export function defaultColorRange(): MaskComponentKind {
  return {
    kind: 'color_range',
    sample_rgb: [0.5, 0.5, 0.5],
    tolerance: 0.1,
    softness: 0.05
  };
}

export function makeComponent(kind: MaskComponentKind, mode: MaskComponentMode = 'add'): MaskComponent {
  return {
    id: nextId(),
    enabled: true,
    mode,
    invert: false,
    kind,
    source: 'manual'
  };
}

export function makeLayer(name: string, index: number, kind: MaskComponentKind = defaultLinear()): MaskLayer {
  return {
    id: nextId(),
    name,
    enabled: true,
    color: defaultMaskColor(index),
    amount: 1,
    components: [makeComponent(kind)],
    edits: {}
  };
}

export function makeGeneratedLayer(
  name: string,
  index: number,
  rasterId: string,
  generated: GeneratedMeta,
  invert = false
): MaskLayer {
  const layer = makeLayer(name, index, defaultBrush(rasterId));
  return {
    ...layer,
    components: layer.components.map((c): MaskComponent => ({
      ...c,
      invert,
      source: 'generated',
      generated
    }))
  };
}

export function nextLayerName(layers: MaskLayer[]): string {
  let i = layers.length + 1;
  const taken = new Set(layers.map((l) => l.name));
  while (taken.has(`Mask ${i}`)) i++;
  return `Mask ${i}`;
}

const DEDICATED_KIND_CLASSES: Record<string, MaskKind> = {
  sky: 'sky',
  person: 'people'
};

export function visibleSceneClasses<T extends { id: string }>(
  classes: T[],
  kinds: { kind: MaskKind }[]
): T[] {
  return classes.filter((c) => {
    const dedicated = DEDICATED_KIND_CLASSES[c.id];
    return !dedicated || !kinds.some((k) => k.kind === dedicated);
  });
}

export interface MaskCapacity {
  layersFull: boolean;
  componentsFull: boolean;
  totalFull: boolean;
}

export function maskCapacity(edits: Edits, layerId: string | null): MaskCapacity {
  const total = edits.masks.reduce((n, l) => n + l.components.length, 0);
  const layer = layerId ? edits.masks.find((l) => l.id === layerId) ?? null : null;
  return {
    layersFull: edits.masks.length >= N_MAX_MASK_LAYERS,
    componentsFull: layer ? layer.components.length >= N_MAX_COMPONENTS_PER_LAYER : false,
    totalFull: total >= N_MAX_TOTAL_COMPONENTS
  };
}

export function cloneLayerWithNewIds(layer: MaskLayer, color: string, name: string): MaskLayer {
  return {
    id: nextId(),
    name,
    enabled: layer.enabled,
    color,
    amount: layer.amount,
    components: layer.components.map((c) => ({ ...c, id: nextId(), kind: cloneKind(c.kind) })),
    edits: { ...layer.edits }
  };
}

function cloneKind(k: MaskComponentKind): MaskComponentKind {
  if (k.kind === 'linear') {
    return { kind: 'linear', p0: { ...k.p0 }, p1: { ...k.p1 }, feather: k.feather };
  }
  if (k.kind === 'radial') {
    return {
      kind: 'radial',
      center: { ...k.center },
      radius_xy: { ...k.radius_xy },
      feather: k.feather
    };
  }
  if (k.kind === 'brush') return { kind: 'brush', raster_id: k.raster_id };
  if (k.kind === 'luma_range') {
    return { kind: 'luma_range', min: k.min, max: k.max, softness: k.softness };
  }
  if (k.kind === 'polygon') {
    return {
      kind: 'polygon',
      points: k.points.map((p) => ({ ...p })),
      feather: k.feather
    };
  }
  return {
    kind: 'color_range',
    sample_rgb: [...k.sample_rgb],
    tolerance: k.tolerance,
    softness: k.softness
  };
}

export function setMaskedEdit(edits: MaskedEdits, key: MaskedEditKey, value: number): MaskedEdits {
  const next = { ...edits };
  if (value === 0 || Number.isNaN(value)) {
    delete next[key];
  } else {
    next[key] = value;
  }
  return next;
}

export type { Vec2f };
