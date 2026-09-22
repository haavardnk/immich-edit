import { editor } from '$lib/stores/editor.svelte';
import { scopes } from '$lib/stores/scopes.svelte';
import { ui } from '$lib/stores/ui.svelte';
import { openDevelopPanels, SCOPES_PANEL } from './registry';

export function setDevelopPanel(id: string, open: boolean): void {
  const next = openDevelopPanels(ui.developOpenPanels);
  if (open) {
    next.add(id);
  } else {
    next.delete(id);
  }
  ui.setDevelopPanels([...next]);
  if (id !== SCOPES_PANEL) return;
  scopes.setPanelOpen(open);
  if (scopes.needsRender) editor.refreshScopes();
}
