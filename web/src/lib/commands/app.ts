import { ui } from '#lib/stores/ui.svelte.ts';
import type { Command } from '#lib/commands/types.ts';

export const APP_COMMANDS: readonly Command[] = [
  {
    id: 'app.help',
    contexts: ['global'],
    bind: 'help',
    gate: 'outsideFields',
    run: () => ui.toggleKeybindsHelp()
  }
];
