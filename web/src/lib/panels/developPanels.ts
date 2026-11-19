import { modifiedDevelopPanels } from '#lib/editor/modified.ts';
import { revealSetting } from '#lib/editor/revealSetting.ts';
import type { Setting } from '#lib/editor/settingsSearch.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { scopes } from '#lib/stores/scopes.svelte.ts';
import { settingSearch } from '#lib/stores/settingSearch.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
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

export function revealDevelopSetting(setting: Setting): void {
  ui.openTab('develop');
  if (!settingSearch.shows(setting.id)) settingSearch.clear();
  if (ui.developModifiedOnly && !modifiedDevelopPanels(editor.edits).has(setting.panel)) {
    ui.developModifiedOnly = false;
  }
  setDevelopPanel(setting.panel, true);
  void revealSetting(setting.id, { focus: true, fallback: `[data-panel="${setting.panel}"]` });
}
