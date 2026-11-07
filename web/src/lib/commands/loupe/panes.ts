import { canDropPane, dropFocusedPane } from '#lib/browse/loupeActions.ts';
import { advanceFocused } from '#lib/browse/loupeNav.ts';
import { loupeCommand } from '#lib/commands/loupe/context.ts';
import { forward } from '#lib/commands/loupe/navigation.ts';
import type { Command, IconCommand } from '#lib/commands/types.ts';
import { compare } from '#lib/stores/compare.svelte.ts';
import { mdiArrowCollapseLeft, mdiCheckboxMarkedCircleOutline, mdiClose, mdiLink } from '@mdi/js';

const paneSync = loupeCommand({
  id: 'compare.sync',
  contexts: ['compare', 'survey'],
  label: 'Sync zoom and pan',
  short: 'Sync',
  icon: mdiLink,
  bind: 'paneSync',
  pressed: () => compare.syncView,
  run: () => (compare.syncView = !compare.syncView)
});

export const panePromote = loupeCommand({
  id: 'compare.promote',
  contexts: ['compare'],
  label: 'Make select',
  short: 'Promote',
  icon: mdiArrowCollapseLeft,
  bind: 'panePromote',
  enabled: () => compare.focusIndex > 0,
  run: () => compare.promote(compare.focusIndex)
});

const surveyKeep = loupeCommand({
  id: 'survey.keep',
  contexts: ['survey'],
  label: 'Keep only this photo',
  short: 'Keep',
  icon: mdiCheckboxMarkedCircleOutline,
  bind: 'surveyKeep',
  run: () => compare.keepOnly(compare.focusIndex)
});

export const paneDrop = loupeCommand({
  id: 'compare.drop',
  contexts: ['compare', 'survey'],
  label: 'Drop this photo',
  short: 'Drop',
  icon: mdiClose,
  bind: 'paneDrop',
  enabled: canDropPane,
  run: dropFocusedPane
});

export const PANE_ACTIONS: readonly IconCommand[] = [paneSync, panePromote, surveyKeep, paneDrop];

export const LOUPE_PANE_COMMANDS: readonly Command[] = [
  loupeCommand({
    id: 'compare.swap',
    contexts: ['compare', 'survey'],
    bind: 'paneSwap',
    run: (trigger) => advanceFocused(forward(trigger))
  }),
  ...PANE_ACTIONS
];
