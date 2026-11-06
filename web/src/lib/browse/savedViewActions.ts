import { matchBrowseRoute } from '#lib/browse/browseRoutes.ts';
import { activeFilterValues } from '#lib/browse/filterChips.ts';
import { withFilterParams } from '#lib/browse/filterParams.ts';
import { idName } from '#lib/browse/idOptions.ts';
import { savedViewName } from '#lib/browse/savedViewName.ts';
import { parseSavedViewUrl, savedViewPath, savedViewUrl } from '#lib/browse/savedViewUrl.ts';
import { browseControls, type SortFamily } from '#lib/stores/browseControls.svelte.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { goto } from '$app/navigation';
import { page } from '$app/state';

function familyOf(url: URL): SortFamily | null {
  return matchBrowseRoute(url)?.family ?? null;
}

export function currentViewUrl(): string | null {
  const canonical = withFilterParams(page.url, browseControls.filters);
  const sorted = familyOf(canonical) !== null;
  return savedViewUrl(
    canonical.pathname + canonical.search,
    sorted ? browseControls.sort : null,
    browseView.gridSize
  );
}

export function currentViewName(title: string): string {
  return savedViewName(activeFilterValues(browseControls.filters, idName), title);
}

export function isCurrentView(url: string): boolean {
  const target = parseSavedViewUrl(url);
  return (
    target !== null &&
    savedViewPath(target.path) === savedViewPath(page.url.pathname + page.url.search)
  );
}

export async function openSavedView(url: string): Promise<void> {
  const target = parseSavedViewUrl(url);
  if (!target) {
    toasts.push('error', 'This saved view has an invalid address');
    return;
  }
  if (target.size) browseView.setGridSize(target.size);
  const family = familyOf(new URL(target.path, page.url.href));
  const current = isCurrentView(url);
  if (target.sort && family) {
    if (current) browseControls.applySort(target.sort, family);
    else browseControls.queueSort(target.sort, family);
  }
  if (current) return;
  try {
    await goto(target.path);
    browseControls.settleQueuedSort(true);
  } catch (e) {
    browseControls.settleQueuedSort(false);
    toasts.fail('Failed to open view', e);
  }
}
