import {
  clearFlags,
  setLabel,
  setRating,
  toggleFavorite,
  toggleReject,
  type MetaOutcome,
  type MetaTarget
} from '#lib/browse/assetMeta.ts';
import type { CullKind } from '#lib/browse/cullMark.ts';
import { cullState, type CullActions } from '#lib/browse/cullState.ts';
import { advanceFocused, stepLoupe } from '#lib/browse/loupeNav.ts';
import { ratingHud } from '#lib/browse/ratingHud.svelte.ts';
import { editorHref } from '#lib/editor/navigation.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { compare } from '#lib/stores/compare.svelte.ts';
import type { LabelColor } from '#lib/stores/labels.ts';
import { loupeTags } from '#lib/stores/loupeTags.svelte.ts';
import { recentTags } from '#lib/stores/recentTags.svelte.ts';
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
  if (compare.mode === 'single') {
    if (browseView.loupeId === id) stepLoupe(1);
  } else if (compare.focusedId === id) advanceFocused(1);
}

interface CullMark {
  kind: CullKind;
  advance: boolean;
}

function announce(target: MetaTarget, kind: CullKind, advance: boolean): void {
  const asset = target.read();
  if (!asset || (kind !== 'rating' && !advance)) return;
  const step = advance ? () => advanceFrom(target.id) : null;
  ratingHud.show(target.id, kind, cullState(asset), step);
}

function cull(write: (target: MetaTarget) => Promise<MetaOutcome>, mark?: CullMark): void {
  const id = loupeFocusedId();
  if (!id) return;
  const target = loupeTags.target(id);
  let announced = false;
  void write({
    ...target,
    patch: (fields) => {
      target.patch(fields);
      if (announced || !mark) return;
      announced = true;
      announce(target, mark.kind, mark.advance);
    }
  });
}

function marked(kind: CullKind): CullMark {
  return { kind, advance: browseView.loupeAutoAdvance };
}

export function rateFocused(value: number | null): void {
  cull((target) => setRating(target, value), marked('rating'));
}

export function rateFocusedAndAdvance(value: number | null): void {
  cull((target) => setRating(target, value), { kind: 'rating', advance: true });
}

export function favoriteFocused(): void {
  cull(toggleFavorite, marked('favorite'));
}

export function rejectFocused(): void {
  cull(toggleReject, marked('reject'));
}

export function unflagFocused(): void {
  cull(clearFlags);
}

export function labelFocused(color: LabelColor | null): void {
  cull((target) => setLabel(target, color), marked('label'));
}

export const loupeCullActions: CullActions = {
  rate: rateFocused,
  favorite: favoriteFocused,
  reject: rejectFocused,
  label: labelFocused
};

export function tagFocusedRecent(slot: number): void {
  const id = loupeFocusedId();
  const tag = recentTags.tags[slot];
  if (id && tag) void loupeTags.add(id, tag);
}

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
