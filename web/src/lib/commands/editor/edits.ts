import type { Command } from '#lib/commands/types.ts';
import { createVirtualCopy } from '#lib/editor/copies.ts';
import { hasTextSelection } from '#lib/shortcuts/targets.ts';
import { NO_EDITS_TO_COPY, NOTHING_COPIED } from '#lib/stores/clipboard.svelte.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { mdiAutoFix, mdiContentCopy, mdiContentPaste, mdiRestore } from '@mdi/js';
import { editorAssetId, returnPath } from '#lib/commands/editor/context.ts';

export const autoAdjust = {
  id: 'editor.auto',
  contexts: ['editor'],
  label: 'Auto adjust',
  icon: mdiAutoFix,
  bind: 'autoAdjust',
  enabled: () => !editor.autoBusy && !!editor.assetId,
  run: () => void editor.onAutoAdjust()
} satisfies Command;

export const resetGeometry = {
  id: 'editor.resetGeometry',
  contexts: ['editor'],
  label: 'Reset geometry',
  icon: mdiRestore,
  bind: 'resetEdits',
  active: () => ui.editorTab === 'geometry',
  enabled: () => editor.canResetGeometry,
  run: () => void editor.resetGeometry()
} satisfies Command;

export const resetDevelop = {
  id: 'editor.resetDevelop',
  contexts: ['editor'],
  label: 'Reset develop settings',
  icon: mdiRestore,
  bind: 'resetEdits',
  enabled: () => editor.canResetDevelop,
  run: () => void editor.onReset()
} satisfies Command;

export const copyEdits = {
  id: 'editor.copy',
  contexts: ['editor'],
  label: 'Copy edits',
  icon: mdiContentCopy,
  bind: 'copyEdits',
  yields: hasTextSelection,
  enabled: () => !!editor.assetId && editor.hasEdits,
  unavailable: () => (editor.hasEdits ? null : NO_EDITS_TO_COPY),
  run: () => editor.copyEdits()
} satisfies Command;

export const pasteEdits = {
  id: 'editor.paste',
  contexts: ['editor'],
  label: 'Paste edits',
  icon: mdiContentPaste,
  bind: 'pasteEdits',
  yields: hasTextSelection,
  enabled: () => editor.canPaste,
  unavailable: () => (editor.hasClipboard ? null : NOTHING_COPIED),
  run: () => void editor.pasteEdits()
} satisfies Command;

export const EDIT_COMMANDS: readonly Command[] = [
  { id: 'editor.undo', contexts: ['editor'], bind: 'undo', run: () => editor.undo() },
  { id: 'editor.redo', contexts: ['editor'], bind: 'redo', run: () => editor.redo() },
  autoAdjust,
  resetGeometry,
  resetDevelop,
  copyEdits,
  pasteEdits,
  {
    id: 'editor.virtualCopy',
    contexts: ['editor'],
    bind: 'createVirtualCopy',
    run: () => {
      const id = editorAssetId();
      if (id) void createVirtualCopy(id, { returnPath: returnPath() });
    }
  }
];
