import { loupeFocusedId } from '#lib/browse/loupeActions.ts';
import { copyLocationFrom, pasteLocationTo } from '#lib/browse/locationCopy.ts';
import { loupeCommand, withFocused } from '#lib/commands/loupe/context.ts';
import type { Command } from '#lib/commands/types.ts';

export const LOUPE_LOCATION_COMMANDS: readonly Command[] = [
  loupeCommand({
    id: 'loupe.copyLocation',
    contexts: ['loupe'],
    bind: 'copyLocation',
    yields: () => !loupeFocusedId(),
    run: withFocused((id) => void copyLocationFrom(id))
  }),
  loupeCommand({
    id: 'loupe.pasteLocation',
    contexts: ['loupe'],
    bind: 'pasteLocation',
    yields: () => !loupeFocusedId(),
    run: withFocused((id) => void pasteLocationTo([id]))
  })
];
