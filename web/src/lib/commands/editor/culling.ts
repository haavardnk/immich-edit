import { nextRatingFromKey } from '#lib/browse/ratingShortcuts.ts';
import type { CullActions } from '#lib/browse/cullState.ts';
import { trashLabelFor } from '#lib/browse/trash.ts';
import { editorAssetId, leaveEditor } from '#lib/commands/editor/context.ts';
import type { Command } from '#lib/commands/types.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { labelOf, nextLabelFromKey } from '#lib/stores/labels.ts';
import { trash } from '#lib/stores/trash.svelte.ts';
import type { AssetDetail } from '#lib/types/asset.ts';
import { mdiTrashCanOutline } from '@mdi/js';

function trashTarget(): AssetDetail | null {
  const id = editorAssetId();
  const asset = editor.asset;
  return id !== null && asset?.id === id ? asset : null;
}

async function trashAndLeave(): Promise<void> {
  const asset = trashTarget();
  if (!asset || !(await trash.request([asset]))) return;
  if (editorAssetId() === asset.id) leaveEditor(asset.id);
}

export const editorCullActions: CullActions = {
  rate: (value) => void editor.setRating(value),
  favorite: () => void editor.toggleFavorite(),
  reject: () => void editor.toggleReject(),
  label: (color) => void editor.setLabel(color)
};

export const editorTrash = {
  id: 'editor.trash',
  contexts: ['editor'],
  label: () => trashLabelFor(editorAssetId()),
  icon: mdiTrashCanOutline,
  enabled: () => trashTarget() !== null,
  run: () => void trashAndLeave()
} satisfies Command;

export const CULLING_COMMANDS: readonly Command[] = [
  {
    id: 'editor.favorite',
    contexts: ['editor'],
    bind: 'favorite',
    run: editorCullActions.favorite
  },
  {
    id: 'editor.reject',
    contexts: ['editor'],
    bind: 'reject',
    run: editorCullActions.reject
  },
  {
    id: 'editor.unflag',
    contexts: ['editor'],
    bind: 'unflag',
    run: () => void editor.clearFlags()
  },
  editorTrash,
  {
    id: 'editor.label',
    contexts: ['editor'],
    bind: 'label',
    run: (trigger) => {
      if (!trigger) return;
      const next = nextLabelFromKey(trigger.key, editor.asset ? labelOf(editor.asset) : null);
      if (next !== undefined) editorCullActions.label(next);
    }
  },
  {
    id: 'editor.rate',
    contexts: ['editor'],
    bind: 'rate',
    run: (trigger) => {
      if (!trigger) return;
      const next = nextRatingFromKey(trigger.key, editor.asset?.exifInfo?.rating ?? null);
      if (next !== undefined) editorCullActions.rate(next);
    }
  }
];
