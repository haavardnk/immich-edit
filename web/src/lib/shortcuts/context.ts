import { ui } from '#lib/stores/ui.svelte.ts';
import { compare } from '#lib/stores/compare.svelte.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { onEditorRoute, onViewerRoute } from '#lib/utils/route.ts';
import type { KeybindContext } from './table';
import { CONTEXT_STACKS } from './stacks';

export function activeContexts(): readonly KeybindContext[] {
  if (onViewerRoute()) return [];
  if (onEditorRoute()) {
    const tab = ui.editorTab;
    if (tab === 'masks' || tab === 'retouch' || tab === 'geometry') return CONTEXT_STACKS[tab];
    return CONTEXT_STACKS.editor;
  }
  if (compare.mode !== 'single') return CONTEXT_STACKS[compare.mode];
  if (browseView.loupeId) return CONTEXT_STACKS.loupe;
  return CONTEXT_STACKS.grid;
}
