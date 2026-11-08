import { surveyUnavailable } from '#lib/browse/compareEntry.ts';
import { gridCurrent, openGridEditor, openMulti } from '#lib/browse/gridActions.ts';
import { peek } from '#lib/browse/peek.svelte.ts';
import { gridCommand } from '#lib/commands/grid/context.ts';
import type { GridHost } from '#lib/commands/grid/gridHost.ts';
import type { Command } from '#lib/commands/types.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { compare } from '#lib/stores/compare.svelte.ts';

function withCurrent(host: GridHost, open: (id: string) => void): void {
  const id = gridCurrent(host.items());
  peek.hide();
  if (id) open(id);
}

function peekTarget(host: GridHost): string | null {
  return peek.hovered ?? gridCurrent(host.items());
}

function nothingCurrent(host: GridHost): boolean {
  return gridCurrent(host.items()) === null;
}

export const GRID_NAVIGATION_COMMANDS: readonly Command[] = [
  gridCommand({
    id: 'grid.openLoupe',
    bind: 'openLoupe',
    yields: nothingCurrent,
    run: (host) => withCurrent(host, (id) => browseView.openLoupe(id))
  }),
  gridCommand({
    id: 'grid.openEditor',
    bind: 'openEditor',
    yields: nothingCurrent,
    run: (host) => withCurrent(host, openGridEditor)
  }),
  gridCommand({
    id: 'grid.peek',
    bind: 'peek',
    yields: (host) => peekTarget(host) === null,
    run: (host, trigger) => {
      const id = peekTarget(host);
      if (id && !trigger?.repeat) peek.show(id);
    },
    release: () => peek.hide()
  }),
  gridCommand({
    id: 'grid.compare',
    bind: 'enterCompare',
    run: (host) => openMulti('compare', host.items())
  }),
  gridCommand({
    id: 'grid.survey',
    bind: 'enterSurvey',
    shown: () => compare.surveyAvailable,
    unavailable: surveyUnavailable,
    run: (host) => openMulti('survey', host.items())
  })
];
