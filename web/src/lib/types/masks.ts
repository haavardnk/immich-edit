import {
  N_MAX_COMPONENTS_PER_LAYER,
  N_MAX_MASK_LAYERS,
  N_MAX_TOTAL_COMPONENTS,
  type Edits,
  type MaskComponent,
  type MaskComponentKind,
  type MaskComponentMode,
  type MaskLayer,
  type MaskedEditKey,
  type MaskedEdits,
  type Vec2f
} from './edits';

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
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return `id-${Math.random().toString(36).slice(2)}-${Date.now().toString(36)}`;
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

export function makeComponent(kind: MaskComponentKind, mode: MaskComponentMode = 'add'): MaskComponent {
  return {
    id: nextId(),
    enabled: true,
    mode,
    opacity: 1,
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

export function nextLayerName(layers: MaskLayer[]): string {
  let i = layers.length + 1;
  const taken = new Set(layers.map((l) => l.name));
  while (taken.has(`Mask ${i}`)) i++;
  return `Mask ${i}`;
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
  return { kind: 'brush', raster_id: k.raster_id };
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
