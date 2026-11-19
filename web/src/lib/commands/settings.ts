import { createHost } from '#lib/commands/host.ts';
import type { Command } from '#lib/commands/types.ts';

export interface SettingsSearchHost {
  focus: () => void;
}

export const settingsSearchHost = createHost<SettingsSearchHost>();

export const SETTINGS_COMMANDS: readonly Command[] = [
  {
    id: 'settings.search',
    contexts: ['global'],
    bind: 'settingsSearch',
    gate: 'outsideFields',
    active: () => settingsSearchHost.get() !== null,
    run: () => settingsSearchHost.get()?.focus()
  }
];
