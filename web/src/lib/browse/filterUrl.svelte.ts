import { goto } from '$app/navigation';
import { page } from '$app/state';
import { untrack } from 'svelte';
import { browseControls, FILTER_DEFAULTS } from '$lib/stores/browseControls.svelte';
import { isTypedEdit, readFilterParams, withFilterParams } from './filterParams';
import { matchBrowseRoute } from './browseRoutes';

function navigate(url: URL, replaceState: boolean): void {
  void goto(url, { replaceState, keepFocus: true, noScroll: true });
}

export function syncFilterUrl(ready: () => boolean): void {
  let route: string | null = null;

  $effect(() => {
    const url = page.url;
    if (!ready()) return;
    untrack(() => {
      if (!matchBrowseRoute(url)) {
        route = null;
        return;
      }
      const filters = readFilterParams(url);
      const canonical = withFilterParams(url, filters);
      if (canonical.search !== url.search) navigate(canonical, true);
      const bare = withFilterParams(url, FILTER_DEFAULTS);
      const moved = bare.pathname + bare.search !== route;
      route = bare.pathname + bare.search;
      if (moved) return;
      if (withFilterParams(url, browseControls.filters).search !== canonical.search) {
        browseControls.applyFilters(filters);
      }
    });
  });

  $effect(() => {
    const filters = browseControls.filters;
    const corrected = browseControls.takeCorrected();
    const url = untrack(() => page.url);
    const next = withFilterParams(url, filters);
    untrack(() => {
      if (!ready() || !matchBrowseRoute(url) || next.search === url.search) return;
      navigate(next, corrected || isTypedEdit(readFilterParams(url), filters));
    });
  });
}
