import { leaveMulti } from '#lib/browse/loupeNav.ts';
import { loupeCommand, VIEWING } from '#lib/commands/loupe/context.ts';
import type { Command } from '#lib/commands/types.ts';
import type { KeybindContext, KeybindId } from '#lib/shortcuts/table.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { compare } from '#lib/stores/compare.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { mdiArrowLeft } from '@mdi/js';

function exitFullscreen(id: string, context: KeybindContext, bind: KeybindId): Command {
  return loupeCommand({
    id,
    contexts: [context],
    label: 'Exit fullscreen',
    bind,
    gate: 'always',
    active: (trigger) => ui.fullscreen && trigger.key === 'Escape',
    run: () => ui.toggleFullscreen()
  });
}

export const loupeClose = loupeCommand({
  id: 'loupe.close',
  contexts: ['loupe'],
  label: 'Back',
  icon: mdiArrowLeft,
  bind: 'loupeClose',
  run: () => browseView.closeLoupe()
});

export const LOUPE_DISMISS_COMMANDS: readonly Command[] = [
  exitFullscreen('loupe.exitFullscreen', 'loupe', 'loupeClose'),
  exitFullscreen('compare.exitFullscreen', 'compare', 'compareExit'),
  exitFullscreen('survey.exitFullscreen', 'survey', 'surveyExit'),
  loupeClose,
  loupeCommand({
    id: 'compare.exit',
    contexts: ['compare'],
    bind: 'compareExit',
    run: leaveMulti
  }),
  loupeCommand({
    id: 'survey.exit',
    contexts: ['survey'],
    bind: 'surveyExit',
    run: leaveMulti
  }),
  loupeCommand({
    id: 'loupe.backToGrid',
    contexts: VIEWING,
    bind: 'backToGrid',
    run: () => {
      if (compare.mode !== 'single') leaveMulti();
      browseView.closeLoupe();
    }
  })
];
