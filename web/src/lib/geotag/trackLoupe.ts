import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { trackSession } from '#lib/stores/trackSession.svelte.ts';

export function openTrackPhoto(id: string): void {
  const photos = [...trackSession.photos];
  if (!photos.some((asset) => asset.id === id)) return;
  browsing.set(photos);
  browsing.total = photos.length;
  browseView.openLoupe(id);
}
