import { onEditorRoute } from '#lib/utils/route.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { gridHost } from '#lib/commands/grid/gridHost.ts';
import type { Command } from '#lib/commands/types.ts';

export const APP_COMMANDS: readonly Command[] = [
  {
    id: 'app.help',
    contexts: ['global'],
    bind: 'help',
    gate: 'outsideFields',
    active: () => onEditorRoute() || browseView.loupeId !== null || gridHost.get() !== null,
    enabled: () => !gridHost.get()?.selectingAll(),
    run: () => ui.toggleKeybindsHelp()
  }
];
