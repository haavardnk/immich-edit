import {
  MASK_COLOR_TOKENS,
  MASKED_EDIT_KEYS,
  type ClickPointMeta,
  type Edits,
  type GeneratedMeta,
  type MaskComponent,
  type MaskComponentKind,
  type MaskLayer,
  type MaskedEdits,
  type RangeMeta,
  type Vec2f
} from '$lib/types/edits';

export function decodeMasks(ops: Record<string, unknown>, edits: Edits): void {
  const masks = ops.masks as { layers?: unknown[] } | undefined;
  if (!masks?.layers) return;
  edits.masks = masks.layers
    .map((raw) => parseMaskLayer(raw))
    .filter((layer): layer is MaskLayer => layer !== null);
}

export function parseVec2f(raw: unknown): Vec2f | null {
  if (!raw || typeof raw !== 'object') return null;
  const record = raw as Record<string, unknown>;
  if (typeof record.x !== 'number' || typeof record.y !== 'number') return null;
  return { x: record.x, y: record.y };
}

function parseMaskLayer(raw: unknown): MaskLayer | null {
  if (!raw || typeof raw !== 'object') return null;
  const record = raw as Record<string, unknown>;
  if (typeof record.id !== 'string') return null;
  const componentsRaw = Array.isArray(record.components) ? record.components : [];
  const components: MaskComponent[] = [];
  for (const component of componentsRaw) {
    const parsed = parseMaskComponent(component);
    if (parsed) components.push(parsed);
    else return null;
  }
  const edits: MaskedEdits = {};
  const editsRaw = (record.edits ?? {}) as Record<string, unknown>;
  for (const key of MASKED_EDIT_KEYS) {
    const value = editsRaw[key];
    if (typeof value === 'number') edits[key] = value;
  }
  return {
    id: record.id,
    name: typeof record.name === 'string' ? record.name : '',
    enabled: record.enabled !== false,
    color: typeof record.color === 'string' ? record.color : MASK_COLOR_TOKENS[0],
    amount: typeof record.amount === 'number' ? record.amount : 1,
    invert: record.invert === true,
    components,
    edits
  };
}

function parseMaskComponent(raw: unknown): MaskComponent | null {
  if (!raw || typeof raw !== 'object') return null;
  const record = raw as Record<string, unknown>;
  if (typeof record.id !== 'string') return null;
  const kind = parseMaskKind(record.kind);
  if (!kind) return null;
  const mode = record.mode === 'subtract' || record.mode === 'intersect' ? record.mode : 'add';
  const generated = parseGeneratedMeta(record.generated);
  return {
    id: record.id,
    enabled: record.enabled !== false,
    mode,
    invert: record.invert === true,
    kind,
    source: record.source === 'generated' ? 'generated' : 'manual',
    ...(generated ? { generated } : {})
  };
}

function parseGeneratedMeta(raw: unknown): GeneratedMeta | undefined {
  if (!raw || typeof raw !== 'object') return undefined;
  const record = raw as Record<string, unknown>;
  if (typeof record.model_id !== 'string' || typeof record.kind !== 'string') return undefined;
  if (typeof record.prob_raster_id !== 'string') return undefined;
  const points = parseClickPoints(record.points);
  const range = parseRangeMeta(record.range);
  return {
    model_id: record.model_id,
    kind: record.kind,
    prob_raster_id: record.prob_raster_id,
    ...(typeof record.class === 'string' ? { class: record.class } : {}),
    grow: typeof record.grow === 'number' ? record.grow : 0,
    feather: typeof record.feather === 'number' ? record.feather : 0,
    ...(record.painted === true ? { painted: true } : {}),
    ...(points.length > 0 ? { points } : {}),
    ...(range ? { range } : {})
  };
}

function parseRangeMeta(raw: unknown): RangeMeta | undefined {
  if (!raw || typeof raw !== 'object') return undefined;
  const record = raw as Record<string, unknown>;
  if (typeof record.min !== 'number' || typeof record.max !== 'number') return undefined;
  return {
    min: record.min,
    max: record.max,
    softness: typeof record.softness === 'number' ? record.softness : 0
  };
}

function parseClickPoints(raw: unknown): ClickPointMeta[] {
  if (!Array.isArray(raw)) return [];
  const points: ClickPointMeta[] = [];
  for (const item of raw) {
    if (!item || typeof item !== 'object') continue;
    const point = item as Record<string, unknown>;
    if (typeof point.x !== 'number' || typeof point.y !== 'number') continue;
    points.push({ x: point.x, y: point.y, positive: point.positive !== false });
  }
  return points;
}

function parseRgb(raw: unknown): [number, number, number] | null {
  if (!Array.isArray(raw) || raw.length !== 3) return null;
  if (!raw.every((value) => typeof value === 'number' && Number.isFinite(value))) return null;
  return [raw[0], raw[1], raw[2]];
}

function parseMaskKind(raw: unknown): MaskComponentKind | null {
  if (!raw || typeof raw !== 'object') return null;
  const record = raw as Record<string, unknown>;
  if (record.kind === 'linear') {
    const p0 = parseVec2f(record.p0);
    const p1 = parseVec2f(record.p1);
    if (!p0 || !p1) return null;
    return {
      kind: 'linear',
      p0,
      p1,
      feather: typeof record.feather === 'number' ? record.feather : 0
    };
  }
  if (record.kind === 'radial') {
    const center = parseVec2f(record.center);
    const radius = parseVec2f(record.radius_xy);
    if (!center || !radius) return null;
    const angle =
      typeof record.angle === 'number' && Number.isFinite(record.angle) && record.angle !== 0
        ? { angle: record.angle }
        : {};
    return {
      kind: 'radial',
      center,
      radius_xy: radius,
      feather: typeof record.feather === 'number' ? record.feather : 0,
      ...angle
    };
  }
  if (record.kind === 'brush') {
    if (typeof record.raster_id !== 'string') return null;
    return { kind: 'brush', raster_id: record.raster_id };
  }
  if (record.kind === 'luma_range') {
    if (
      typeof record.min !== 'number' ||
      typeof record.max !== 'number' ||
      typeof record.softness !== 'number'
    )
      return null;
    return {
      kind: 'luma_range',
      min: record.min,
      max: record.max,
      softness: record.softness
    };
  }
  if (record.kind === 'color_range') {
    const sampleRgb = parseRgb(record.sample_rgb);
    if (!sampleRgb || typeof record.tolerance !== 'number' || typeof record.softness !== 'number')
      return null;
    return {
      kind: 'color_range',
      sample_rgb: sampleRgb,
      tolerance: record.tolerance,
      softness: record.softness
    };
  }
  if (record.kind === 'polygon') {
    if (!Array.isArray(record.points)) return null;
    const points: Vec2f[] = [];
    for (const rawPoint of record.points) {
      const point = rawPoint as Record<string, unknown>;
      if (typeof point?.x !== 'number' || typeof point?.y !== 'number') return null;
      points.push({ x: point.x, y: point.y });
    }
    return {
      kind: 'polygon',
      points,
      feather: typeof record.feather === 'number' ? record.feather : 0
    };
  }
  return null;
}
