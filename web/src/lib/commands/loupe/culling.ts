import {
  favoriteFocused,
  labelFocused,
  loupeAsset,
  loupeFocusedId,
  rateFocused,
  rateFocusedAndAdvance,
  rejectFocused,
  trashFocused,
  unflagFocused
} from '#lib/browse/loupeActions.ts';
import { nextRatingFromKey, ratingFromCode } from '#lib/browse/ratingShortcuts.ts';
import { loupeCommand, VIEWING } from '#lib/commands/loupe/context.ts';
import type { Command, CommandTrigger } from '#lib/commands/types.ts';
import { labelOf, nextLabelFromKey, type LabelColor } from '#lib/stores/labels.ts';

function nextLabel(trigger: CommandTrigger): LabelColor | null | undefined {
  const asset = loupeAsset();
  return nextLabelFromKey(trigger.key, asset ? labelOf(asset) : null);
}

function nextRating(trigger: CommandTrigger): number | null | undefined {
  return nextRatingFromKey(trigger.key, loupeAsset()?.exifInfo?.rating ?? 0);
}

function advanceRating(trigger: CommandTrigger): number | null | undefined {
  return loupeFocusedId() ? ratingFromCode(trigger.code) : undefined;
}

function keyed<T>(
  read: (trigger: CommandTrigger) => T | undefined,
  apply: (value: T) => void
): Pick<Command, 'yields' | 'run'> {
  return {
    yields: (trigger) => read(trigger) === undefined,
    run: (trigger) => {
      const value = trigger && read(trigger);
      if (value !== undefined) apply(value);
    }
  };
}

export const LOUPE_CULLING_COMMANDS: readonly Command[] = [
  loupeCommand({ id: 'loupe.favorite', contexts: VIEWING, bind: 'favorite', run: favoriteFocused }),
  loupeCommand({ id: 'loupe.reject', contexts: VIEWING, bind: 'reject', run: rejectFocused }),
  loupeCommand({ id: 'loupe.unflag', contexts: VIEWING, bind: 'unflag', run: unflagFocused }),
  loupeCommand({ id: 'loupe.trash', contexts: ['loupe'], bind: 'trash', run: trashFocused }),
  loupeCommand({
    id: 'loupe.label',
    contexts: VIEWING,
    bind: 'label',
    ...keyed(nextLabel, labelFocused)
  }),
  loupeCommand({
    id: 'loupe.rate',
    contexts: VIEWING,
    bind: 'rate',
    ...keyed(nextRating, rateFocused)
  }),
  loupeCommand({
    id: 'loupe.rateAdvance',
    contexts: VIEWING,
    bind: 'rateAdvance',
    ...keyed(advanceRating, rateFocusedAndAdvance)
  })
];
