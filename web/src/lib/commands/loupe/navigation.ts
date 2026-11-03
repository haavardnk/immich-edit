import { openInEditor } from '#lib/browse/loupeActions.ts';
import { paneColumns } from '#lib/browse/loupeLayout.ts';
import { enterMulti, openLastLoupe, stepLoupe } from '#lib/browse/loupeNav.ts';
import { loupeCommand } from '#lib/commands/loupe/context.ts';
import type { Command, CommandTrigger } from '#lib/commands/types.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { compare } from '#lib/stores/compare.svelte.ts';

function toEdge(trigger?: CommandTrigger): void {
  if (trigger?.key === 'Home') {
    const first = browsing.visible[0];
    if (first) browseView.openLoupe(first.id);
    return;
  }
  void openLastLoupe();
}

function surveyStep(key: string): number {
  if (key === 'ArrowRight') return 1;
  if (key === 'ArrowLeft') return -1;
  const cols = paneColumns(compare.members.length);
  return key === 'ArrowDown' ? cols : -cols;
}

export function forward(trigger?: CommandTrigger): number {
  return trigger?.key === 'ArrowRight' ? 1 : -1;
}

export const LOUPE_NAVIGATION_COMMANDS: readonly Command[] = [
  loupeCommand({
    id: 'loupe.step',
    contexts: ['loupe'],
    bind: 'loupeNav',
    run: (trigger) => stepLoupe(forward(trigger))
  }),
  loupeCommand({ id: 'loupe.edge', contexts: ['loupe'], bind: 'loupeEdge', run: toEdge }),
  loupeCommand({
    id: 'compare.focus',
    contexts: ['compare'],
    bind: 'compareFocus',
    run: (trigger) => compare.focusDelta(forward(trigger))
  }),
  loupeCommand({
    id: 'survey.focus',
    contexts: ['survey'],
    bind: 'surveyFocus',
    run: (trigger) => compare.focusDelta(surveyStep(trigger?.key ?? ''))
  }),
  loupeCommand({
    id: 'compare.focusCycle',
    contexts: ['compare', 'survey'],
    bind: 'paneFocusCycle',
    run: (trigger) => compare.focusDelta(trigger?.shiftKey ? -1 : 1)
  }),
  loupeCommand({
    id: 'loupe.compare',
    contexts: ['loupe'],
    bind: 'enterCompare',
    run: () => enterMulti('compare')
  }),
  loupeCommand({
    id: 'loupe.survey',
    contexts: ['loupe'],
    bind: 'enterSurvey',
    run: () => enterMulti('survey')
  }),
  loupeCommand({
    id: 'loupe.openEditor',
    contexts: ['loupe'],
    bind: 'openEditor',
    run: () => openInEditor()
  }),
  loupeCommand({
    id: 'compare.openEditor',
    contexts: ['compare', 'survey'],
    bind: 'paneOpenEditor',
    run: () => openInEditor()
  })
];
