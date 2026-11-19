import {
  browseTarget,
  clearFlags,
  setLabel,
  setRating,
  toggleFavorite,
  toggleReject
} from '#lib/browse/assetMeta.ts';
import { multiMembers, type MultiMode } from '#lib/browse/compareEntry.ts';
import { nextRatingFromKey } from '#lib/browse/ratingShortcuts.ts';
import { editorHref } from '#lib/editor/navigation.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { compare, MAX_PANES } from '#lib/stores/compare.svelte.ts';
import { labelOf, nextLabelFromKey, type LabelColor } from '#lib/stores/labels.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { trash } from '#lib/stores/trash.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { currentPath } from '#lib/utils/route.ts';
import { goto } from '$app/navigation';

export function gridTargets(): string[] {
  return [...selection.selected];
}

export function gridCurrent(items: readonly AssetSummary[]): string | null {
  const id = browseView.activeId;
  if (id && selection.has(id)) return id;
  return items.find((asset) => selection.has(asset.id))?.id ?? null;
}

function pickedAssets(items: readonly AssetSummary[]): AssetSummary[] {
  return items.filter((asset) => selection.has(asset.id));
}

function common<T>(values: readonly T[]): T | undefined {
  return values.every((value) => value === values[0]) ? values[0] : undefined;
}

export function rateTargets(rating: number | null): void {
  gridTargets().forEach((id) => void setRating(browseTarget(id), rating));
}

export function favoriteTargets(): void {
  gridTargets().forEach((id) => void toggleFavorite(browseTarget(id)));
}

export function rejectTargets(): void {
  gridTargets().forEach((id) => void toggleReject(browseTarget(id)));
}

export function unflagTargets(): void {
  gridTargets().forEach((id) => void clearFlags(browseTarget(id)));
}

export function labelTargets(color: LabelColor | null): void {
  gridTargets().forEach((id) => void setLabel(browseTarget(id), color));
}

export function trashTargets(items: readonly AssetSummary[]): void {
  void trash.request(pickedAssets(items));
}

export function nextTargetLabel(
  items: readonly AssetSummary[],
  key: string
): LabelColor | null | undefined {
  if (gridTargets().length === 0) return undefined;
  const current = common(pickedAssets(items).map((asset) => labelOf(asset))) ?? null;
  return nextLabelFromKey(key, current);
}

export function nextTargetRating(
  items: readonly AssetSummary[],
  key: string
): number | null | undefined {
  const current = common(pickedAssets(items).map((asset) => asset.exifInfo?.rating ?? null));
  return nextRatingFromKey(key, current);
}

export function rateAndAdvance(
  items: readonly AssetSummary[],
  rating: number | null,
  reveal: (id: string) => void
): void {
  const ids = gridTargets();
  const [only] = ids;
  if (ids.length > 1 || !only) {
    rateTargets(rating);
    return;
  }
  const following = items[items.findIndex((asset) => asset.id === only) + 1];
  void setRating(browseTarget(only), rating).then((outcome) => {
    if (outcome === 'skipped' || !following) return;
    selection.replace([following.id]);
    browseView.setActive(following.id);
    reveal(following.id);
  });
}

export function openMulti(
  mode: MultiMode,
  items: readonly AssetSummary[],
  panes: number = MAX_PANES
): void {
  const members = multiMembers(
    mode,
    items.map((asset) => asset.id),
    selection.selected,
    gridCurrent(items),
    panes
  );
  const first = members[0];
  if (members.length < 2 || !first) {
    toasts.push('info', `${mode} needs two photos`);
    return;
  }
  browseView.openLoupe(first);
  compare.enter(mode, members);
}

export function openGridEditor(id: string): void {
  void goto(editorHref(id, currentPath()));
}
