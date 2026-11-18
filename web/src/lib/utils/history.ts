import type { EditHistoryEntry } from '#lib/api/edits.ts';
import { neutralEdits, type DevelopSection, type Edits } from '#lib/types/edits.ts';
import {
  fieldChanged,
  fieldDelta,
  formatField,
  HISTORY_FIELDS,
  historyGroupLabel
} from '#lib/utils/historyFields.ts';
import {
  curveItems,
  dcpLabel,
  geometryItems,
  lensBlurItems,
  maskItems,
  profileChanged,
  whiteBalanceLabel,
  type HistoryDetailItem
} from '#lib/utils/historySummaries.ts';

export type HistoryDetailGroup = {
  key: string;
  label: string;
  items: HistoryDetailItem[];
};

export type HistoryLabel = {
  label: string;
  delta?: string;
};

function snapshots(entry: EditHistoryEntry, previous: EditHistoryEntry | null): [Edits, Edits] {
  const prev = previous && !previous.deleted && previous.edits ? previous.edits : neutralEdits();
  const curr = !entry.deleted && entry.edits ? entry.edits : neutralEdits();
  return [prev, curr];
}

function groupFor(
  groups: Map<DevelopSection, HistoryDetailGroup>,
  key: DevelopSection
): HistoryDetailGroup {
  const existing = groups.get(key);
  if (existing) return existing;
  const group: HistoryDetailGroup = { key, label: historyGroupLabel(key), items: [] };
  groups.set(key, group);
  return group;
}

export function historyDetails(
  entry: EditHistoryEntry,
  previous: EditHistoryEntry | null,
  isRaw = true
): HistoryDetailGroup[] {
  const [prev, curr] = snapshots(entry, previous);
  const groups = new Map<DevelopSection, HistoryDetailGroup>();
  const wbBefore = whiteBalanceLabel(prev.basic.white_balance);
  const wbAfter = whiteBalanceLabel(curr.basic.white_balance);
  if (wbBefore !== wbAfter) {
    groupFor(groups, 'white_balance').items.push({
      kind: 'value',
      label: 'White Balance',
      before: wbBefore,
      after: wbAfter
    });
  }
  for (const field of HISTORY_FIELDS) {
    if (!fieldChanged(field, prev, curr, isRaw)) continue;
    groupFor(groups, field.section).items.push({
      kind: 'value',
      label: field.label,
      before: formatField(field, prev, isRaw),
      after: formatField(field, curr, isRaw)
    });
  }
  const blurItems = lensBlurItems(prev.lens_blur, curr.lens_blur);
  if (blurItems.length > 0) groupFor(groups, 'lens_blur').items.push(...blurItems);
  if (profileChanged(prev.lens, curr.lens)) {
    groupFor(groups, 'lens').items.push({ kind: 'summary', text: 'Profile data changed' });
  }
  if (
    prev.color.dcp.mode !== curr.color.dcp.mode ||
    prev.color.dcp.profile_id !== curr.color.dcp.profile_id ||
    prev.color.dcp.illuminant !== curr.color.dcp.illuminant
  ) {
    groupFor(groups, 'dcp').items.push({
      kind: 'summary',
      text: `Camera profile: ${dcpLabel(prev)} → ${dcpLabel(curr)}`
    });
  }
  if (prev.color.lut_3d.lut_id !== curr.color.lut_3d.lut_id) {
    groupFor(groups, 'lut').items.push({ kind: 'summary', text: 'LUT selection changed' });
  }
  const curves = curveItems(prev, curr);
  if (curves.length > 0) groupFor(groups, 'curves').items.push(...curves);
  const nested: HistoryDetailGroup[] = [
    { key: 'masks', label: 'Masks', items: maskItems(prev.masks, curr.masks) },
    { key: 'geometry', label: 'Geometry', items: geometryItems(prev.geometry, curr.geometry) }
  ];
  return [...groups.values(), ...nested.filter((group) => group.items.length > 0)];
}

export function historyLabel(
  entry: EditHistoryEntry,
  previous: EditHistoryEntry | null,
  isRaw = true
): HistoryLabel {
  if (entry.deleted) return { label: entry.action ?? 'Reset to original' };
  const curr = entry.edits;
  if (!curr) return { label: entry.action ?? entry.manifest_hash.slice(0, 8) };

  const [prev] = snapshots(entry, previous);
  const scalarDiffs = HISTORY_FIELDS.filter((field) => fieldChanged(field, prev, curr, isRaw));
  const details = historyDetails(entry, previous, isRaw);
  const detailCount = details.reduce((count, group) => count + group.items.length, 0);
  const only = scalarDiffs.length === 1 ? scalarDiffs[0] : null;
  if (only && detailCount === 1) {
    const delta = fieldDelta(only, prev, curr, isRaw);
    return delta === undefined ? { label: only.label } : { label: only.label, delta };
  }
  if (entry.action) return { label: entry.action };
  if (detailCount === 0) return { label: entry.manifest_hash.slice(0, 8) };
  return { label: 'Multiple changes' };
}
