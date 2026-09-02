import {
  bandsAllZero,
  colorGradeIsZero,
  curvesAreIdentity,
  curvesEditsIsIdentity,
  dcpIsDefault,
  decodeFlatOps,
  encodeFlatOps,
  HSL_BANDS,
  isFullCrop,
  lut3dIsActive,
  MASK_COLOR_TOKENS,
  MASKED_EDIT_KEYS,
  MAX_RETOUCH_POINTS,
  MAX_RETOUCH_STROKES,
  neutralEdits,
  type AspectLock,
  type ClickPointMeta,
  type ColorGradeRegion,
  type CropRect,
  type CurvePoint,
  type DcpEdits,
  type EditManifest,
  type Edits,
  type GeneratedMeta,
  type GeometryEdits,
  type HslBand,
  type MaskComponent,
  type MaskComponentKind,
  type MaskLayer,
  type MaskedEdits,
  type RangeMeta,
  type RetouchStroke,
  type Vec2f
} from '$lib/types/edits';
import {
  clampPerspective,
  neutralPerspective,
  perspectiveIsIdentity,
  type PerspectiveEdits
} from '$lib/utils/perspective';

export function editsToManifest(edits: Edits): EditManifest {
  const ops: Record<string, unknown> = {};
  encodeFlatOps(edits, ops);
  if (!curvesEditsIsIdentity(edits.basic.curves)) {
    const obj: Record<string, [number, number][]> = {};
    const curves = edits.basic.curves;
    if (!curvesAreIdentity(curves.composite))
      obj.composite = curves.composite.map((point) => [point.x, point.y]);
    if (!curvesAreIdentity(curves.r)) obj.r = curves.r.map((point) => [point.x, point.y]);
    if (!curvesAreIdentity(curves.g)) obj.g = curves.g.map((point) => [point.x, point.y]);
    if (!curvesAreIdentity(curves.b)) obj.b = curves.b.map((point) => [point.x, point.y]);
    if (!curvesAreIdentity(curves.luma)) obj.luma = curves.luma.map((point) => [point.x, point.y]);
    ops.curves = obj;
  }
  if (!bandsAllZero(edits.color.hsl.bands))
    ops.hsl = {
      bands: edits.color.hsl.bands.map((band) => ({
        hue: band.hue,
        sat: band.sat,
        lum: band.lum
      }))
    };
  if (!colorGradeIsZero(edits.color.color_grade)) {
    const colorGrade = edits.color.color_grade;
    const region = (value: ColorGradeRegion) => ({
      hue: value.hue,
      sat: value.sat,
      lum: value.lum
    });
    ops.color_grade = {
      shadows: region(colorGrade.shadows),
      midtones: region(colorGrade.midtones),
      highlights: region(colorGrade.highlights),
      global: region(colorGrade.global),
      balance: colorGrade.balance,
      blend: colorGrade.blend
    };
  }
  if (lut3dIsActive(edits.color.lut_3d))
    ops.lut_3d = {
      lut_id: edits.color.lut_3d.lut_id,
      amount: edits.color.lut_3d.amount
    };
  if (!dcpIsDefault(edits.color.dcp))
    ops.dcp_hue_sat = {
      mode: edits.color.dcp.mode,
      profile_id: edits.color.dcp.profile_id,
      illuminant: edits.color.dcp.illuminant,
      use_tone_curve: edits.color.dcp.use_tone_curve,
      use_base_table: edits.color.dcp.use_base_table,
      use_look_table: edits.color.dcp.use_look_table,
      use_baseline_exposure: edits.color.dcp.use_baseline_exposure
    };
  const cropActive = !isFullCrop(edits.geometry.crop);
  const angleActive = Math.abs(edits.geometry.rotate_angle) > 1e-4;
  const aspectActive = edits.geometry.aspect.kind !== 'original';
  const rotateActive = edits.geometry.rotate !== 0;
  const flipActive = edits.geometry.flip_h || edits.geometry.flip_v;
  const perspectiveActive = !perspectiveIsIdentity(edits.geometry.perspective);
  if (
    cropActive ||
    angleActive ||
    aspectActive ||
    rotateActive ||
    flipActive ||
    perspectiveActive
  ) {
    const obj: Record<string, unknown> = {};
    if (rotateActive) obj.rotate = edits.geometry.rotate;
    if (edits.geometry.flip_h) obj.flip_h = true;
    if (edits.geometry.flip_v) obj.flip_v = true;
    if (angleActive) obj.angle = edits.geometry.rotate_angle;
    if (edits.geometry.crop && cropActive) obj.crop = edits.geometry.crop;
    if (perspectiveActive && edits.geometry.perspective)
      obj.perspective = clampPerspective(edits.geometry.perspective);
    obj.aspect = edits.geometry.aspect;
    ops.transform = obj;
  }
  if (edits.masks.length > 0) ops.masks = { layers: edits.masks };
  if (edits.retouch.length > 0) ops.retouch = { strokes: edits.retouch };
  return { schema_version: 4, ops };
}

export function manifestToEdits(doc: EditManifest): Edits {
  const edits = neutralEdits();
  const ops = doc.ops ?? {};
  decodeFlatOps(ops, edits);
  if ((doc.schema_version ?? 0) < 4 && edits.lens.profile_enabled === null) {
    edits.lens.profile_enabled = false;
  }
  const curves = ops.curves as
    | {
        points?: number[][];
        composite?: number[][];
        r?: number[][];
        g?: number[][];
        b?: number[][];
        luma?: number[][];
      }
    | undefined;
  if (curves) {
    const decode = (points: number[][] | undefined): CurvePoint[] | null => {
      if (!points || points.length < 2) return null;
      return points.map((point) => ({ x: point[0], y: point[1] }));
    };
    if (curves.points) {
      const legacy = decode(curves.points);
      if (legacy) edits.basic.curves.composite = legacy;
    } else {
      const composite = decode(curves.composite);
      if (composite) edits.basic.curves.composite = composite;
      const red = decode(curves.r);
      if (red) edits.basic.curves.r = red;
      const green = decode(curves.g);
      if (green) edits.basic.curves.g = green;
      const blue = decode(curves.b);
      if (blue) edits.basic.curves.b = blue;
      const luma = decode(curves.luma);
      if (luma) edits.basic.curves.luma = luma;
    }
  }
  const hsl = ops.hsl as { bands?: HslBand[] } | undefined;
  if (hsl?.bands) {
    for (let index = 0; index < HSL_BANDS && index < hsl.bands.length; index++) {
      const band = hsl.bands[index];
      if (band.hue !== undefined) edits.color.hsl.bands[index].hue = band.hue;
      if (band.sat !== undefined) edits.color.hsl.bands[index].sat = band.sat;
      if (band.lum !== undefined) edits.color.hsl.bands[index].lum = band.lum;
    }
  }
  const colorGrade = ops.color_grade as
    | {
        shadows?: ColorGradeRegion;
        midtones?: ColorGradeRegion;
        highlights?: ColorGradeRegion;
        global?: ColorGradeRegion;
        balance?: number;
        blend?: number;
      }
    | undefined;
  if (colorGrade) {
    const readRegion = (source: ColorGradeRegion | undefined, target: ColorGradeRegion): void => {
      if (!source) return;
      if (source.hue !== undefined) target.hue = source.hue;
      if (source.sat !== undefined) target.sat = source.sat;
      if (source.lum !== undefined) target.lum = source.lum;
    };
    readRegion(colorGrade.shadows, edits.color.color_grade.shadows);
    readRegion(colorGrade.midtones, edits.color.color_grade.midtones);
    readRegion(colorGrade.highlights, edits.color.color_grade.highlights);
    readRegion(colorGrade.global, edits.color.color_grade.global);
    if (colorGrade.balance !== undefined) edits.color.color_grade.balance = colorGrade.balance;
    if (colorGrade.blend !== undefined) edits.color.color_grade.blend = colorGrade.blend;
  }
  const lut3d = ops.lut_3d as { lut_id?: string; amount?: number } | undefined;
  if (lut3d?.lut_id !== undefined) edits.color.lut_3d.lut_id = lut3d.lut_id;
  if (lut3d?.amount !== undefined) edits.color.lut_3d.amount = lut3d.amount;
  const dcp = ops.dcp_hue_sat as Partial<DcpEdits> | undefined;
  if (dcp) {
    if (dcp.mode !== undefined) edits.color.dcp.mode = dcp.mode;
    if (dcp.profile_id !== undefined) edits.color.dcp.profile_id = dcp.profile_id;
    if (dcp.illuminant !== undefined) edits.color.dcp.illuminant = dcp.illuminant;
    if (dcp.use_tone_curve !== undefined) edits.color.dcp.use_tone_curve = dcp.use_tone_curve;
    if (dcp.use_base_table !== undefined) edits.color.dcp.use_base_table = dcp.use_base_table;
    if (dcp.use_look_table !== undefined) edits.color.dcp.use_look_table = dcp.use_look_table;
    if (dcp.use_baseline_exposure !== undefined)
      edits.color.dcp.use_baseline_exposure = dcp.use_baseline_exposure;
  }
  const transform = ops.transform as
    | {
        rotate?: number;
        flip_h?: boolean;
        flip_v?: boolean;
        angle?: number;
        crop?: CropRect;
        aspect?: AspectLock;
        perspective?: PerspectiveEdits;
      }
    | undefined;
  if (transform?.rotate !== undefined)
    edits.geometry.rotate = transform.rotate as GeometryEdits['rotate'];
  if (transform?.flip_h !== undefined) edits.geometry.flip_h = transform.flip_h;
  if (transform?.flip_v !== undefined) edits.geometry.flip_v = transform.flip_v;
  if (transform?.angle !== undefined) edits.geometry.rotate_angle = transform.angle;
  if (transform?.crop) edits.geometry.crop = transform.crop;
  if (transform?.aspect) edits.geometry.aspect = transform.aspect;
  if (transform?.perspective) {
    const perspective = clampPerspective({ ...neutralPerspective(), ...transform.perspective });
    edits.geometry.perspective = perspectiveIsIdentity(perspective) ? null : perspective;
  }
  const masks = ops.masks as { layers?: unknown[] } | undefined;
  if (masks?.layers) {
    edits.masks = masks.layers
      .map((raw) => parseMaskLayer(raw))
      .filter((layer): layer is MaskLayer => layer !== null);
  }
  const retouch = ops.retouch as { strokes?: unknown[] } | undefined;
  if (retouch?.strokes) {
    edits.retouch = retouch.strokes
      .map((raw) => parseRetouchStroke(raw))
      .filter((stroke): stroke is RetouchStroke => stroke !== null)
      .slice(0, MAX_RETOUCH_STROKES);
  }
  return edits;
}

function parseRetouchStroke(raw: unknown): RetouchStroke | null {
  if (!raw || typeof raw !== 'object') return null;
  const record = raw as Record<string, unknown>;
  if (typeof record.id !== 'string') return null;
  const source = parseVec2f(record.source);
  if (!source) return null;
  const points = (Array.isArray(record.points) ? record.points : [])
    .map((point) => parseVec2f(point))
    .filter((point): point is Vec2f => point !== null)
    .slice(0, MAX_RETOUCH_POINTS);
  if (points.length === 0) return null;
  return {
    id: record.id,
    mode: record.mode === 'clone' ? 'clone' : 'heal',
    points,
    radius: typeof record.radius === 'number' ? record.radius : 0,
    hardness: typeof record.hardness === 'number' ? record.hardness : 0.5,
    opacity: typeof record.opacity === 'number' ? record.opacity : 1,
    source,
    enabled: record.enabled !== false
  };
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

function parseVec2f(raw: unknown): Vec2f | null {
  if (!raw || typeof raw !== 'object') return null;
  const record = raw as Record<string, unknown>;
  if (typeof record.x !== 'number' || typeof record.y !== 'number') return null;
  return { x: record.x, y: record.y };
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
    return {
      kind: 'radial',
      center,
      radius_xy: radius,
      feather: typeof record.feather === 'number' ? record.feather : 0
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
