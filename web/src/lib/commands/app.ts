import { gridHost } from '#lib/commands/grid/gridHost.ts';
import type { Command } from '#lib/commands/types.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { onEditorRoute } from '#lib/utils/route.ts';
import { mdiCircleHalfFull, mdiKeyboardOutline } from '@mdi/js';

export const help = {
  id: 'app.help',
  contexts: ['global'],
  label: 'Keyboard shortcuts',
  icon: mdiKeyboardOutline,
  bind: 'help',
  gate: 'outsideFields',
  active: () => onEditorRoute() || browseView.loupeId !== null || gridHost.get() !== null,
  enabled: () => !gridHost.get()?.selectingAll(),
  run: () => ui.toggleKeybindsHelp()
} satisfies Command;

export const greyCanvas = {
  id: 'app.greyCanvas',
  contexts: ['global'],
  label: 'Grey background',
  icon: mdiCircleHalfFull,
  pressed: () => ui.greyCanvas,
  run: () => ui.toggleGreyCanvas()
} satisfies Command;

export const APP_COMMANDS: readonly Command[] = [help, greyCanvas];
