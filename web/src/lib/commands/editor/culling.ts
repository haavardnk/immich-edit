import { nextRatingFromKey } from '#lib/browse/ratingShortcuts.ts';
import type { Command } from '#lib/commands/types.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { labelOf, nextLabelFromKey } from '#lib/stores/labels.ts';

export const CULLING_COMMANDS: readonly Command[] = [
  {
    id: 'editor.favorite',
    contexts: ['editor'],
    bind: 'favorite',
    run: () => void editor.toggleFavorite()
  },
  {
    id: 'editor.reject',
    contexts: ['editor'],
    bind: 'reject',
    run: () => void editor.toggleReject()
  },
  {
    id: 'editor.unflag',
    contexts: ['editor'],
    bind: 'unflag',
    run: () => void editor.clearFlags()
  },
  {
    id: 'editor.label',
    contexts: ['editor'],
    bind: 'label',
    run: (trigger) => {
      if (!trigger) return;
      const next = nextLabelFromKey(trigger.key, editor.asset ? labelOf(editor.asset) : null);
      if (next !== undefined) void editor.setLabel(next);
    }
  },
  {
    id: 'editor.rate',
    contexts: ['editor'],
    bind: 'rate',
    run: (trigger) => {
      if (!trigger) return;
      const next = nextRatingFromKey(trigger.key, editor.asset?.exifInfo?.rating ?? null);
      if (next !== undefined) void editor.setRating(next);
    }
  }
];
