import type { EditHistoryEntry } from '#lib/api/edits.ts';
import type { HistoryDetailGroup } from '#lib/utils/history.ts';
import {
  mdiBlurRadial,
  mdiCameraIris,
  mdiCircleHalfFull,
  mdiCropRotate,
  mdiImageOutline,
  mdiLayersTripleOutline,
  mdiPaletteOutline,
  mdiTuneVariant,
  mdiViewGridOutline,
  mdiWhiteBalanceSunny
} from '@mdi/js';

const GROUP_ICONS: Record<string, string> = {
  tone: mdiWhiteBalanceSunny,
  curves: mdiWhiteBalanceSunny,
  white_balance: mdiPaletteOutline,
  hsl: mdiPaletteOutline,
  color_grading: mdiPaletteOutline,
  bw: mdiPaletteOutline,
  lut: mdiPaletteOutline,
  dcp: mdiPaletteOutline,
  presence: mdiCircleHalfFull,
  vignette: mdiCircleHalfFull,
  grain: mdiCircleHalfFull,
  lens_blur: mdiCircleHalfFull,
  sharpening: mdiBlurRadial,
  noise_reduction: mdiBlurRadial,
  lens: mdiCameraIris,
  masks: mdiLayersTripleOutline,
  geometry: mdiCropRotate
};

export function historyIcon(entry: EditHistoryEntry, groups: HistoryDetailGroup[]): string {
  if (entry.deleted || !entry.edits) return mdiImageOutline;
  if (entry.action?.startsWith('Preset')) return mdiViewGridOutline;
  const icons = new Set(groups.map((group) => GROUP_ICONS[group.key] ?? mdiTuneVariant));
  const [only] = icons;
  return icons.size === 1 && only ? only : mdiTuneVariant;
}
