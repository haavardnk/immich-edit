import { backToGrid } from '#lib/browse/backToGrid.ts';
import { layout } from '#lib/device/layout.svelte.ts';
import { editorHref } from '#lib/editor/navigation.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import type { VersionStore } from '#lib/stores/editor/versions.svelte.ts';
import { returnPath } from '#lib/utils/route.ts';
import { goto } from '$app/navigation';

export function versionHref(id: string): string {
  return editorHref(id, returnPath());
}

export async function deleteVersion(
  versions: VersionStore,
  id: string,
  onleave: () => void = () => {}
): Promise<boolean> {
  const master = versions.master;
  const wasCurrent = versions.current?.id === id;
  if (!(await versions.remove(id))) return false;
  if (!wasCurrent || !master) return true;
  onleave();
  await goto(versionHref(master));
  return true;
}

export async function openVersionInLoupe(id: string): Promise<void> {
  await backToGrid(id, returnPath(), layout.mode === 'touch' ? 'return' : 'push');
  if (browsing.visible.some((asset) => asset.id === id)) browseView.openLoupe(id);
}
