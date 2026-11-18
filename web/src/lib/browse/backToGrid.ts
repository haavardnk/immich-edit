import { goto } from '$app/navigation';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { navHistory } from '#lib/stores/navHistory.ts';
import { validReturnPath } from '#lib/editor/navigation.ts';

export type GridExit = 'push' | 'return';

export async function backToGrid(
  assetId: string | null,
  returnPath?: string | null,
  exit: GridExit = 'push'
): Promise<void> {
  const returnToLoupe = browseView.consumeEditorReturnLoupe(assetId);
  const inList = assetId !== null && browsing.visible.some((a) => a.id === assetId);
  browseView.setActive(inList ? assetId : null);
  const target = validReturnPath(returnPath) ? returnPath : (browseView.lastGridPath ?? '/photos');
  await (exit === 'return' ? navHistory.returnTo(target) : goto(target));
  if (returnToLoupe && inList && assetId) browseView.openLoupe(assetId);
}
