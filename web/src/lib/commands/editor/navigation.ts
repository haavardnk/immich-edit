import { editorHref } from '#lib/editor/navigation.ts';
import type { KeybindId } from '#lib/shortcuts/table.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { ui, type EditorTab } from '#lib/stores/ui.svelte.ts';
import { goto } from '$app/navigation';
import { page } from '$app/state';
import type { Command } from '#lib/commands/types.ts';
import { editorAssetId, leaveEditor, selectedShapeNudges } from '#lib/commands/editor/context.ts';
import { returnPath } from '#lib/utils/route.ts';
import { mdiArrowLeft } from '@mdi/js';
import { layout } from '#lib/device/layout.svelte.ts';

function openInEditor(id: string): void {
  void goto(editorHref(id, returnPath()), { replace: true });
}

function navigate(id: string, delta: number): void {
  const target = delta < 0 ? browsing.prevOf(id) : browsing.nextOf(id);
  if (target) {
    browsing.prefetchNear(target.id);
    openInEditor(target.id);
    return;
  }
  const request = delta < 0 ? browsing.requestPrevious() : browsing.requestMore();
  void request.then((loaded) => {
    if (!loaded || page.params.id !== id) return;
    const next = delta < 0 ? browsing.prevOf(id) : browsing.nextOf(id);
    if (next) openInEditor(next.id);
  });
}

function openTab(tab: EditorTab, bind: KeybindId): Command {
  return { id: `editor.open.${tab}`, contexts: ['editor'], bind, run: () => ui.openTab(tab) };
}

export const editorBack = {
  id: 'editor.backToGrid',
  contexts: ['editor'],
  label: 'Back',
  icon: mdiArrowLeft,
  bind: 'backToGrid',
  run: () => leaveEditor(editorAssetId(), layout.mode === 'touch' ? 'return' : 'push')
} satisfies Command;

export const NAVIGATION_COMMANDS: readonly Command[] = [
  {
    id: 'editor.navigate',
    contexts: ['editor'],
    bind: 'editorNav',
    yields: selectedShapeNudges,
    run: (trigger) => {
      const id = editorAssetId();
      if (id && trigger) navigate(id, trigger.key === 'ArrowLeft' ? -1 : 1);
    }
  },
  editorBack,
  openTab('develop', 'openDevelop'),
  openTab('geometry', 'openGeometry'),
  openTab('retouch', 'openRetouch'),
  openTab('masks', 'openMasks'),
  openTab('export', 'openExport')
];
