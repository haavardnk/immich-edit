import { page } from '$app/state';
import { ui } from '$lib/stores/ui.svelte';
import { compare } from '$lib/stores/compare.svelte';
import { browseView } from '$lib/stores/browseView.svelte';
import type { KeybindContext } from '$lib/keybinds';

export function activeContexts(): KeybindContext[] {
  if (page.url.pathname.startsWith('/assets/')) {
    if (ui.editorTab === 'masks') return ['masks', 'editor', 'global'];
    if (ui.editorTab === 'retouch') return ['retouch', 'editor', 'global'];
    return ['editor', 'global'];
  }
  if (compare.mode !== 'single') return [compare.mode, 'global'];
  if (browseView.loupeId) return ['loupe', 'global'];
  return ['grid', 'global'];
}
