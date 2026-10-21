import { page } from '$app/state';
import { ui } from '$lib/stores/ui.svelte';
import { compare } from '$lib/stores/compare.svelte';
import { browseView } from '$lib/stores/browseView.svelte';
import type { KeybindContext } from './table';

export const CONTEXT_STACKS = {
  masks: ['masks', 'editor', 'global'],
  retouch: ['retouch', 'editor', 'global'],
  geometry: ['geometry', 'editor', 'global'],
  editor: ['editor', 'global'],
  compare: ['compare', 'global'],
  survey: ['survey', 'global'],
  loupe: ['loupe', 'global'],
  grid: ['grid', 'global']
} as const satisfies Record<string, readonly KeybindContext[]>;

export function activeContexts(): readonly KeybindContext[] {
  if (page.url.pathname.startsWith('/assets/')) {
    const tab = ui.editorTab;
    if (tab === 'masks' || tab === 'retouch' || tab === 'geometry') return CONTEXT_STACKS[tab];
    return CONTEXT_STACKS.editor;
  }
  if (compare.mode !== 'single') return CONTEXT_STACKS[compare.mode];
  if (browseView.loupeId) return CONTEXT_STACKS.loupe;
  return CONTEXT_STACKS.grid;
}
