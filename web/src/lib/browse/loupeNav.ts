import { browsing } from '#lib/stores/browsing.svelte.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { compare, type CompareMode } from '#lib/stores/compare.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { neighbourMembers, type MultiMode } from '#lib/browse/compareEntry.ts';
import { switchMembers } from '#lib/browse/loupeLayout.ts';
import { countOf } from '#lib/utils/plural.ts';

function isMulti(): boolean {
  return compare.mode !== 'single';
}

function loadedIds(): string[] {
  return browsing.visible.map((asset) => asset.id);
}

let stepsPastEdge = 0;

function walk(from: string, steps: number, delta: number): string {
  let cursor = from;
  for (let step = 0; step < steps; step += 1) {
    const next = delta > 0 ? browsing.nextOf(cursor) : browsing.prevOf(cursor);
    if (!next) break;
    cursor = next.id;
  }
  return cursor;
}

export function canStepLoupe(id: string, delta: number): boolean {
  return delta > 0
    ? browsing.nextOf(id) !== null || browsing.hasMore
    : browsing.prevOf(id) !== null || browsing.hasPrevious;
}

export function stepLoupe(delta: number): void {
  const from = browseView.loupeId;
  if (!from) return;
  const next = delta > 0 ? browsing.nextOf(from) : browsing.prevOf(from);
  if (next) {
    stepsPastEdge = 0;
    browseView.openLoupe(next.id);
    return;
  }
  stepsPastEdge += 1;
  if (stepsPastEdge > 1) return;
  const request = delta > 0 ? browsing.requestMore() : browsing.requestPrevious();
  void request.then((loaded) => {
    const steps = stepsPastEdge;
    stepsPastEdge = 0;
    if (!loaded || browseView.loupeId !== from) return;
    const target = walk(from, steps, delta);
    if (target !== from) browseView.openLoupe(target);
  });
}

export async function openLastLoupe(): Promise<void> {
  const from = browseView.loupeId;
  if (!(await browsing.requestEdge('last'))) {
    while (browsing.hasMore && (await browsing.requestMore())) {
      if (browseView.loupeId !== from) return;
    }
  }
  const last = browsing.visible.at(-1);
  if (last && browseView.loupeId === from) browseView.openLoupe(last.id);
}

export async function openFirstLoupe(): Promise<void> {
  const from = browseView.loupeId;
  await browsing.requestEdge('first');
  const first = browsing.visible[0];
  if (first && browseView.loupeId === from) browseView.openLoupe(first.id);
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
  if (survivors.length > 0) selection.replace(survivors);
  if (id) browseView.openLoupe(id);
}

function enterMulti(mode: MultiMode): void {
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

export function dropPane(index: number): void {
  const id = compare.members[index];
  if (id === undefined) return;
  if (compare.members.length > 2) return compare.drop(index);
  browseView.openLoupe(compare.members.find((member) => member !== id) ?? id);
}

function togglePane(id: string): void {
  const at = compare.members.indexOf(id);
  if (at >= 0) return dropPane(at);
  if (compare.full) {
    toasts.push('info', `Survey holds up to ${countOf(compare.paneLimit, 'photo')}`);
    return;
  }
  compare.addMember(id);
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
