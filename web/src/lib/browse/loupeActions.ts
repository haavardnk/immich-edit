import {
  clearFlags,
  setLabel,
  setRating,
  toggleFavorite,
  toggleReject,
  type MetaOutcome,
  type MetaTarget
} from '#lib/browse/assetMeta.ts';
import type { CullActions } from '#lib/browse/cullState.ts';
import { advanceFocused, stepLoupe } from '#lib/browse/loupeNav.ts';
import { editorHref } from '#lib/editor/navigation.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { compare } from '#lib/stores/compare.svelte.ts';
import type { LabelColor } from '#lib/stores/labels.ts';
import { loupeTags } from '#lib/stores/loupeTags.svelte.ts';
import { trash } from '#lib/stores/trash.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { currentPath, isEditorPath } from '#lib/utils/route.ts';
import { goto } from '$app/navigation';

export function loupeFocusedId(): string | null {
  return compare.mode === 'single' ? browseView.loupeId : compare.focusedId;
}

export function loupeAsset(): AssetSummary | null {
  const id = loupeFocusedId();
  return id ? (browsing.assets.find((asset) => asset.id === id) ?? null) : null;
}

export function canDropPane(): boolean {
  return compare.members.length > (compare.mode === 'compare' ? 2 : 1);
}

export function dropFocusedPane(): void {
  if (canDropPane()) compare.drop(compare.focusIndex);
}

function advanceFrom(id: string): void {
  if (compare.mode === 'single') stepLoupe(1);
  else if (compare.focusedId === id) advanceFocused(1);
}

function autoAdvance(id: string): void {
  if (browseView.loupeAutoAdvance) advanceFrom(id);
}

function cull(
  write: (target: MetaTarget) => Promise<MetaOutcome>,
  after?: (id: string) => void
): void {
  const id = loupeFocusedId();
  if (!id) return;
  void write(loupeTags.target(id)).then((outcome) => {
    if (outcome !== 'skipped') after?.(id);
  });
}

export function rateFocused(value: number | null): void {
  cull((target) => setRating(target, value), autoAdvance);
}

export function rateFocusedAndAdvance(value: number | null): void {
  cull((target) => setRating(target, value), advanceFrom);
}

export function favoriteFocused(): void {
  cull(toggleFavorite, autoAdvance);
}

export function rejectFocused(): void {
  cull(toggleReject, autoAdvance);
}

export function unflagFocused(): void {
  cull(clearFlags);
}

export function labelFocused(color: LabelColor | null): void {
  cull((target) => setLabel(target, color), autoAdvance);
}

export const loupeCullActions: CullActions = {
  rate: rateFocused,
  favorite: favoriteFocused,
  reject: rejectFocused,
  label: labelFocused
};

export function trashFocused(): void {
  const asset = loupeAsset();
  if (compare.mode !== 'single' || !asset) return;
  void trash.request([asset]);
}

export function leaveLoupeOnEditorPath(pathname: string): void {
  const id = loupeFocusedId();
  if (id && isEditorPath(pathname)) browseView.leaveLoupeForEditor(id);
}

export function openInEditor(id: string | null = loupeFocusedId()): void {
  if (!id) return;
  browseView.leaveLoupeForEditor(id);
  void goto(editorHref(id, currentPath()));
}
