import { goto } from '$app/navigation';
import { browseView } from '$lib/stores/browseView.svelte';
import { browsing } from '$lib/stores/browsing.svelte';

export async function backToGrid(assetId: string | null): Promise<void> {
  const inList = assetId !== null && browsing.assets.some((a) => a.id === assetId);
  browseView.setActive(inList ? assetId : null);
  await goto(browseView.lastGridPath ?? '/photos');
}
