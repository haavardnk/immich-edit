import { goto } from '$app/navigation';
import { browseView } from '$lib/stores/browseView.svelte';
import { browsing } from '$lib/stores/browsing.svelte';
import { validReturnPath } from '$lib/editorNavigation';

export async function backToGrid(
  assetId: string | null,
  returnPath?: string | null
): Promise<void> {
  const returnToLoupe = browseView.consumeEditorReturnLoupe(assetId);
  const inList = assetId !== null && browsing.assets.some((a) => a.id === assetId);
  browseView.setActive(inList ? assetId : null);
  const target = validReturnPath(returnPath) ? returnPath : (browseView.lastGridPath ?? '/photos');
  await goto(target);
  if (returnToLoupe && assetId) browseView.openLoupe(assetId);
}
