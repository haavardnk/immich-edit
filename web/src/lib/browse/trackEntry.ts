import { goto } from '$app/navigation';
import { resolveAssets } from '#lib/browse/locationOps.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { trackSession } from '#lib/stores/trackSession.svelte.ts';

export async function matchTrackFor(ids: readonly string[]): Promise<void> {
  try {
    trackSession.open(await resolveAssets(ids));
  } catch (error) {
    toasts.fail('Match to a track', error);
    return;
  }
  await goto('/map');
}
