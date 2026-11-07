import { openInEditor } from '#lib/browse/loupeActions.ts';
import { paneColumns, paneWindow } from '#lib/browse/loupeLayout.ts';
import { openFirstLoupe, openLastLoupe, selectViewMode, stepLoupe } from '#lib/browse/loupeNav.ts';
import { loupeCommand } from '#lib/commands/loupe/context.ts';
import type { Command, CommandTrigger } from '#lib/commands/types.ts';
import { compare, type CompareMode } from '#lib/stores/compare.svelte.ts';
import { mdiCompare, mdiImageOutline, mdiPencilOutline, mdiViewGridOutline } from '@mdi/js';

function toEdge(trigger?: CommandTrigger): void {
  void (trigger?.key === 'Home' ? openFirstLoupe() : openLastLoupe());
}

function surveyStep(key: string): number {
  if (key === 'ArrowRight') return 1;
  if (key === 'ArrowLeft') return -1;
  const shown = paneWindow(compare.members.length, compare.focusIndex, compare.paneLimit);
  const cols = paneColumns(shown.end - shown.start);
  return key === 'ArrowDown' ? cols : -cols;
}

export function forward(trigger?: CommandTrigger): number {
  return trigger?.key === 'ArrowRight' ? 1 : -1;
}

function viewMode(mode: CompareMode): Pick<Command, 'pressed' | 'run'> {
  return {
    pressed: () => compare.mode === mode,
    run: () => selectViewMode(mode)
  };
}

export const loupeSingle = loupeCommand({
  id: 'loupe.single',
  contexts: ['loupe'],
  label: 'Single photo',
  short: 'Single',
  icon: mdiImageOutline,
  ...viewMode('single')
});

export const loupeCompare = loupeCommand({
  id: 'loupe.compare',
  contexts: ['loupe'],
  label: 'Compare',
  icon: mdiCompare,
  bind: 'enterCompare',
  ...viewMode('compare')
});

export const loupeSurvey = loupeCommand({
  id: 'loupe.survey',
  contexts: ['loupe'],
  label: 'Survey',
  icon: mdiViewGridOutline,
  bind: 'enterSurvey',
  ...viewMode('survey')
});

export const loupeOpenEditor = loupeCommand({
  id: 'loupe.openEditor',
  contexts: ['loupe'],
  label: 'Edit',
  icon: mdiPencilOutline,
  bind: 'openEditor',
  run: () => openInEditor()
});

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
  loupeSingle,
  loupeCompare,
  loupeSurvey,
  loupeOpenEditor,
  loupeCommand({
    id: 'compare.openEditor',
    contexts: ['compare', 'survey'],
    bind: 'paneOpenEditor',
    run: () => openInEditor()
  })
];
