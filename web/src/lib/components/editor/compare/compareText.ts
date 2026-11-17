import { compareLabel, compareName } from '#lib/editor/compare.ts';
import { editor } from '#lib/stores/editor.svelte.ts';

export function otherVersions(): typeof editor.versions.items {
  return editor.versions.items.filter((version) => version.id !== editor.assetId);
}

export function hasCompareSources(): boolean {
  return editor.snapshots.items.length + otherVersions().length > 0;
}

export function leftName(): string {
  return compareName(editor.compare.source, editor.snapshots.items, editor.versions.items);
}

export function leftLabel(): string {
  return compareLabel(editor.compare.source, editor.snapshots.items, editor.versions.items);
}

export function paneLabels(): { left: string; right: string } {
  if (editor.compare.source.kind === 'original') return { left: 'Original', right: 'Edited' };
  return { left: leftLabel(), right: 'Current' };
}
