import { browsing } from '$lib/stores/browsing.svelte';
import { browseView } from '$lib/stores/browseView.svelte';
import { compare, type CompareMode } from '$lib/stores/compare.svelte';
import { selection } from '$lib/stores/selection.svelte';
import { toasts } from '$lib/stores/toasts.svelte';
import { neighbourMembers, type MultiMode } from '$lib/browse/compareEntry';
import { switchMembers } from '$lib/browse/loupeLayout';

function isMulti(): boolean {
  return compare.mode !== 'single';
}

function loadedIds(): string[] {
  return browsing.assets.map((asset) => asset.id);
}

export function stepLoupe(delta: number): void {
  const from = browseView.loupeId;
  if (!from) return;
  const next = delta > 0 ? browsing.nextOf(from) : browsing.prevOf(from);
  if (next) {
    browseView.openLoupe(next.id);
    return;
  }
  if (delta < 0) return;
  void browsing.requestMore().then((loaded) => {
    const after = loaded && browseView.loupeId === from ? browsing.nextOf(from) : null;
    if (after) browseView.openLoupe(after.id);
  });
}

export async function openLastLoupe(): Promise<void> {
  const from = browseView.loupeId;
  while (browsing.hasMore && (await browsing.requestMore())) {
    if (browseView.loupeId !== from) return;
  }
  const last = browsing.assets.at(-1);
  if (last && browseView.loupeId === from) browseView.openLoupe(last.id);
}

export function advanceFocused(delta: number): void {
  const id = compare.focusedId;
  if (!id) return;
  let cursor = id;
  for (;;) {
    const next = delta > 0 ? browsing.nextOf(cursor) : browsing.prevOf(cursor);
    if (!next) return;
    if (!compare.members.includes(next.id)) {
      compare.setMember(compare.focusIndex, next.id);
      return;
    }
    cursor = next.id;
  }
}

export function leaveMulti(): void {
  const id = compare.focusedId;
  const survivors = compare.mode === 'survey' && compare.pruned ? [...compare.members] : [];
  compare.exit();
  if (survivors.length > 0) selection.selectLoaded(survivors);
  if (id) browseView.openLoupe(id);
}

export function enterMulti(mode: MultiMode): void {
  if (isMulti()) {
    leaveMulti();
    return;
  }
  const members = neighbourMembers(mode, loadedIds(), browseView.loupeId);
  if (members.length < 2) {
    toasts.push('info', `${mode} needs two photos`);
    return;
  }
  compare.enter(mode, members);
}

export function selectViewMode(mode: CompareMode): void {
  if (mode === compare.mode) return;
  if (mode === 'single') {
    leaveMulti();
    return;
  }
  if (!isMulti()) {
    enterMulti(mode);
    return;
  }
  const members = switchMembers(mode, loadedIds(), compare.focusedId, compare.members);
  if (members.length < 2) return;
  compare.enter(mode, members);
}

function togglePane(id: string): void {
  const at = compare.members.indexOf(id);
  if (at < 0) return compare.addMember(id);
  if (compare.members.length > 2) return compare.drop(at);
  browseView.openLoupe(compare.members.find((member) => member !== id) ?? id);
}

export function pickFromStrip(id: string, additive: boolean): void {
  const multi = isMulti();
  if (!additive) {
    if (multi) compare.setMember(compare.focusIndex, id);
    else browseView.openLoupe(id);
    return;
  }
  if (multi) return togglePane(id);
  const currentId = browseView.loupeId;
  if (currentId && id !== currentId) compare.enter('compare', [currentId, id], 1);
}
