import { dropFocusedPane } from '#lib/browse/loupeActions.ts';
import { advanceFocused } from '#lib/browse/loupeNav.ts';
import { loupeCommand } from '#lib/commands/loupe/context.ts';
import { forward } from '#lib/commands/loupe/navigation.ts';
import type { Command } from '#lib/commands/types.ts';
import { compare } from '#lib/stores/compare.svelte.ts';

export const LOUPE_PANE_COMMANDS: readonly Command[] = [
  loupeCommand({
    id: 'compare.swap',
    contexts: ['compare', 'survey'],
    bind: 'paneSwap',
    run: (trigger) => advanceFocused(forward(trigger))
  }),
  loupeCommand({
    id: 'compare.drop',
    contexts: ['compare', 'survey'],
    bind: 'paneDrop',
    run: dropFocusedPane
  }),
  loupeCommand({
    id: 'compare.sync',
    contexts: ['compare', 'survey'],
    bind: 'paneSync',
    run: () => (compare.syncView = !compare.syncView)
  }),
  loupeCommand({
    id: 'compare.promote',
    contexts: ['compare'],
    bind: 'panePromote',
    run: () => compare.promote(compare.focusIndex)
  }),
  loupeCommand({
    id: 'survey.keep',
    contexts: ['survey'],
    bind: 'surveyKeep',
    run: () => compare.keepOnly(compare.focusIndex)
  })
];
