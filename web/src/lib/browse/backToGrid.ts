import { goto } from '$app/navigation';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { navHistory } from '#lib/stores/navHistory.ts';
import { validReturnPath } from '#lib/editor/navigation.ts';
import { zoomOut } from '#lib/browse/loupeZoom.ts';
import { holdPhoto } from '#lib/browse/photoHandoff.ts';

export type GridExit = 'push' | 'return';

export async function backToGrid(
  assetId: string | null,
  returnPath?: string | null,
  exit: GridExit = 'push'
): Promise<void> {
  const behind = browseView.loupeBehindEditor;
  const returnToLoupe = browseView.consumeEditorReturnLoupe(assetId);
  const inList = assetId !== null && browsing.visible.some((a) => a.id === assetId);
  browseView.setActive(inList ? assetId : null);
  const target = validReturnPath(returnPath) ? returnPath : (browseView.lastGridPath ?? '/photos');
  const reopen = returnToLoupe && inList;
  if (behind && !reopen) browseView.closeLoupe();
  if (reopen && assetId) {
    holdPhoto(assetId);
    browseView.openLoupe(assetId);
  }
  await (exit === 'return' ? navHistory.returnTo(target) : goto(target));
  if (!assetId || !inList) return;
  if (!reopen) zoomOut(assetId);
}
