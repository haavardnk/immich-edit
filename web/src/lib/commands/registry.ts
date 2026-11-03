import { APP_COMMANDS } from '#lib/commands/app.ts';
import { CULLING_COMMANDS } from '#lib/commands/editor/culling.ts';
import { DISMISS_COMMANDS } from '#lib/commands/editor/dismiss.ts';
import { EDIT_COMMANDS } from '#lib/commands/editor/edits.ts';
import { NAVIGATION_COMMANDS } from '#lib/commands/editor/navigation.ts';
import { TOOL_COMMANDS } from '#lib/commands/editor/tools.ts';
import { VIEW_COMMANDS } from '#lib/commands/editor/view.ts';
import { createKeyDispatcher } from '#lib/commands/keys.ts';
import type { Command } from '#lib/commands/types.ts';

export const COMMANDS: readonly Command[] = [
  ...APP_COMMANDS,
  ...DISMISS_COMMANDS,
  ...VIEW_COMMANDS,
  ...NAVIGATION_COMMANDS,
  ...EDIT_COMMANDS,
  ...CULLING_COMMANDS,
  ...TOOL_COMMANDS
];

export const commandKeys = createKeyDispatcher(COMMANDS);
