import { goto } from '$app/navigation';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { validReturnPath } from '#lib/editor/navigation.ts';

export async function backToGrid(
  assetId: string | null,
  returnPath?: string | null
): Promise<void> {
  const returnToLoupe = browseView.consumeEditorReturnLoupe(assetId);
  const inList = assetId !== null && browsing.visible.some((a) => a.id === assetId);
  browseView.setActive(inList ? assetId : null);
  const target = validReturnPath(returnPath) ? returnPath : (browseView.lastGridPath ?? '/photos');
  await goto(target);
  if (returnToLoupe && inList && assetId) browseView.openLoupe(assetId);
}
