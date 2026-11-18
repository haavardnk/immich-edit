import { copyEditsFrom, pasteEditsTo } from '#lib/browse/copyPaste.ts';
import { loupeFocusedId } from '#lib/browse/loupeActions.ts';
import { loupeCommand } from '#lib/commands/loupe/context.ts';
import type { Command } from '#lib/commands/types.ts';
import { virtualCopies } from '#lib/stores/virtualCopies.svelte.ts';
import { hasTextSelection } from '#lib/shortcuts/targets.ts';

function editable(): string | null {
  return hasTextSelection() ? null : loupeFocusedId();
}

function withFocused(run: (id: string) => void): () => void {
  return () => {
    const id = loupeFocusedId();
    if (id) run(id);
  };
}

export const LOUPE_EDIT_COMMANDS: readonly Command[] = [
  loupeCommand({
    id: 'loupe.copyEdits',
    contexts: ['loupe'],
    bind: 'copyEdits',
    yields: () => !editable(),
    run: withFocused((id) => void copyEditsFrom(id))
  }),
  loupeCommand({
    id: 'loupe.pasteEdits',
    contexts: ['loupe'],
    bind: 'pasteEdits',
    yields: () => !editable(),
    run: withFocused((id) => void pasteEditsTo([id]))
  }),
  loupeCommand({
    id: 'loupe.virtualCopy',
    contexts: ['loupe'],
    bind: 'createVirtualCopy',
    yields: (trigger) => trigger.repeat || !loupeFocusedId(),
    run: withFocused((id) => void virtualCopies.create([id]))
  })
];
