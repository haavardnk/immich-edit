import {
  favoriteTargets,
  gridTargets,
  labelTargets,
  nextTargetLabel,
  nextTargetRating,
  rateAndAdvance,
  rateTargets,
  rejectTargets,
  trashTargets,
  unflagTargets
} from '#lib/browse/gridActions.ts';
import { ratingFromCode } from '#lib/browse/ratingShortcuts.ts';
import { gridCommand } from '#lib/commands/grid/context.ts';
import type { Command } from '#lib/commands/types.ts';

function noTargets(): boolean {
  return gridTargets().length === 0;
}

export const GRID_CULLING_COMMANDS: readonly Command[] = [
  gridCommand({ id: 'grid.favorite', bind: 'favorite', run: favoriteTargets }),
  gridCommand({ id: 'grid.reject', bind: 'reject', run: rejectTargets }),
  gridCommand({ id: 'grid.unflag', bind: 'unflag', run: unflagTargets }),
  gridCommand({
    id: 'grid.trash',
    bind: 'trash',
    yields: noTargets,
    run: (host) => trashTargets(host.items())
  }),
  gridCommand({
    id: 'grid.label',
    bind: 'label',
    yields: (host, trigger) => nextTargetLabel(host.items(), trigger.key) === undefined,
    run: (host, trigger) => {
      const next = trigger && nextTargetLabel(host.items(), trigger.key);
      if (next !== undefined) labelTargets(next);
    }
  }),
  gridCommand({
    id: 'grid.rate',
    bind: 'rate',
    yields: noTargets,
    run: (host, trigger) => {
      const next = trigger && nextTargetRating(host.items(), trigger.key);
      if (next !== undefined) rateTargets(next);
    }
  }),
  gridCommand({
    id: 'grid.rateAdvance',
    bind: 'rateAdvance',
    yields: (_, trigger) => ratingFromCode(trigger.code) === undefined || noTargets(),
    run: (host, trigger) => {
      const next = trigger && ratingFromCode(trigger.code);
      if (next !== undefined) rateAndAdvance(host.items(), next, host.reveal);
    }
  })
];
