import { compareLabel, compareName } from '#lib/editor/compare.ts';
import { editor } from '#lib/stores/editor.svelte.ts';

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
