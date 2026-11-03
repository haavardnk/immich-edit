import type {
  CurveChannel,
  Edits,
  GeometryEdits,
  LensBlurEdits,
  LensEdits,
  MaskLayer,
  WhiteBalance
} from '#lib/types/edits.ts';
import { CURVE_CHANNELS, isFullCrop, whiteBalanceIsAsShot } from '#lib/types/edits.ts';
import { percentRange } from '#lib/utils/percent.ts';
import { neutralPerspective } from '#lib/utils/perspective.ts';

export type HistoryDetailItem =
  | { kind: 'value'; label: string; before: string; after: string }
  | { kind: 'summary'; text: string };

const CURVE_LABELS: Record<CurveChannel, string> = {
  composite: 'RGB',
  r: 'Red',
  g: 'Green',
  b: 'Blue',
  luma: 'Luma'
};

function curvesEqual(a: { x: number; y: number }[], b: { x: number; y: number }[]): boolean {
  return (
    a.length === b.length &&
    a.every((point, i) => {
      const other = b[i];
      return !!other && Math.abs(point.x - other.x) <= 1e-4 && Math.abs(point.y - other.y) <= 1e-4;
    })
  );
}

export function curveItems(prev: Edits, curr: Edits): HistoryDetailItem[] {
  return CURVE_CHANNELS.flatMap((channel) => {
    const before = prev.basic.curves[channel];
    const after = curr.basic.curves[channel];
    if (curvesEqual(before, after)) return [];
    const suffix =
      before.length === after.length ? 'adjusted' : `${before.length} → ${after.length} points`;
    return [{ kind: 'summary' as const, text: `${CURVE_LABELS[channel]}: ${suffix}` }];
  });
}

export function maskItems(prev: MaskLayer[], curr: MaskLayer[]): HistoryDetailItem[] {
  const prevById = new Map(prev.map((layer) => [layer.id, layer]));
  const currById = new Map(curr.map((layer) => [layer.id, layer]));
  const added = curr.filter((layer) => !prevById.has(layer.id)).length;
  const removed = prev.filter((layer) => !currById.has(layer.id)).length;
  const modified = curr.filter((layer) => {
    const before = prevById.get(layer.id);
    return before !== undefined && JSON.stringify(before) !== JSON.stringify(layer);
  }).length;
  const sameIds = added === 0 && removed === 0;
  const reordered = sameIds && prev.some((layer, i) => layer.id !== curr[i]?.id);
  const changes = [
    added > 0 ? `${added} added` : '',
    removed > 0 ? `${removed} removed` : '',
    modified > 0 ? `${modified} modified` : '',
    reordered ? 'reordered' : ''
  ].filter(Boolean);
  return changes.length > 0 ? [{ kind: 'summary', text: changes.join(', ') }] : [];
}

function aspectLabel(aspect: GeometryEdits['aspect']): string {
  if (aspect.kind === 'ratio') return `${aspect.num}:${aspect.den}`;
  return aspect.kind === 'original' ? 'Original' : 'Free';
}

export function lensBlurItems(prev: LensBlurEdits, curr: LensBlurEdits): HistoryDetailItem[] {
  const items: HistoryDetailItem[] = [];
  if (prev.depth_raster_id !== curr.depth_raster_id)
    items.push({
      kind: 'summary',
      text: curr.depth_raster_id ? 'Depth map generated' : 'Depth map removed'
    });
  const focusBefore = percentRange(prev.focus_min, prev.focus_max);
  const focusAfter = percentRange(curr.focus_min, curr.focus_max);
  if (focusBefore !== focusAfter)
    items.push({ kind: 'value', label: 'Focal Range', before: focusBefore, after: focusAfter });
  if (prev.shape !== curr.shape)
    items.push({ kind: 'value', label: 'Bokeh', before: prev.shape, after: curr.shape });
  return items;
}

export function geometryItems(prev: GeometryEdits, curr: GeometryEdits): HistoryDetailItem[] {
  const items: HistoryDetailItem[] = [];
  if (prev.rotate !== curr.rotate)
    items.push({ kind: 'summary', text: `Rotation: ${prev.rotate}° → ${curr.rotate}°` });
  if (Math.abs(prev.rotate_angle - curr.rotate_angle) > 1e-4) {
    items.push({
      kind: 'summary',
      text: `Angle: ${prev.rotate_angle.toFixed(1)}° → ${curr.rotate_angle.toFixed(1)}°`
    });
  }
  if (prev.flip_h !== curr.flip_h)
    items.push({
      kind: 'summary',
      text: `Horizontal flip: ${prev.flip_h ? 'On' : 'Off'} → ${curr.flip_h ? 'On' : 'Off'}`
    });
  if (prev.flip_v !== curr.flip_v)
    items.push({
      kind: 'summary',
      text: `Vertical flip: ${prev.flip_v ? 'On' : 'Off'} → ${curr.flip_v ? 'On' : 'Off'}`
    });
  if (JSON.stringify(prev.aspect) !== JSON.stringify(curr.aspect)) {
    items.push({
      kind: 'summary',
      text: `Aspect: ${aspectLabel(prev.aspect)} → ${aspectLabel(curr.aspect)}`
    });
  }
  if (JSON.stringify(prev.crop) !== JSON.stringify(curr.crop)) {
    const beforeFull = isFullCrop(prev.crop);
    const afterFull = isFullCrop(curr.crop);
    const text = beforeFull ? 'Crop added' : afterFull ? 'Crop removed' : 'Crop adjusted';
    items.push({ kind: 'summary', text });
  }
  const prevP = prev.perspective ?? neutralPerspective();
  const currP = curr.perspective ?? neutralPerspective();
  for (const key of PERSPECTIVE_KEYS) {
    if (Math.abs(prevP[key] - currP[key]) > 1e-4) {
      items.push({
        kind: 'summary',
        text: `${PERSPECTIVE_LABELS[key]}: ${Math.round(prevP[key])} → ${Math.round(currP[key])}`
      });
    }
  }
  if (JSON.stringify(prevP.corners) !== JSON.stringify(currP.corners)) {
    items.push({ kind: 'summary', text: 'Perspective corners adjusted' });
  }
  return items;
}

const PERSPECTIVE_LABELS = {
  vertical: 'Perspective vertical',
  horizontal: 'Perspective horizontal',
  aspect: 'Perspective aspect'
} as const;

const PERSPECTIVE_KEYS = Object.keys(PERSPECTIVE_LABELS) as (keyof typeof PERSPECTIVE_LABELS)[];

const LENS_PROFILE_KEYS = [
  'k1',
  'k2',
  'k3',
  'vk1',
  'vk2',
  'vk3',
  'ca_red_scale_x10000',
  'ca_blue_scale_x10000'
] as const satisfies readonly (keyof LensEdits)[];

export function profileChanged(prev: LensEdits, curr: LensEdits): boolean {
  return LENS_PROFILE_KEYS.some((key) => Math.abs(prev[key] - curr[key]) > 1e-4);
}

function signedInt(value: number): string {
  const rounded = Math.round(value);
  return rounded > 0 ? `+${rounded}` : String(rounded);
}

export function whiteBalanceLabel(wb: WhiteBalance): string {
  if (whiteBalanceIsAsShot(wb)) return 'As Shot';
  if (wb.mode === 'absolute') return `${Math.round(wb.kelvin)} K, Tint ${signedInt(wb.tint)}`;
  return `Temp ${signedInt(wb.temp)}, Tint ${signedInt(wb.tint)}`;
}

export function dcpLabel(edits: Edits): string {
  if (edits.color.dcp.mode === 'off') return 'Default Color';
  if (edits.color.dcp.mode === 'flat') return 'Flat';
  if (edits.color.dcp.mode === 'auto') return 'Auto';
  return edits.color.dcp.profile_id ? 'Imported profile' : 'Unavailable profile';
}
