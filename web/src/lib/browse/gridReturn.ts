import { browseView } from '#lib/stores/browseView.svelte.ts';

type Located = Pick<URL, 'pathname' | 'search'>;

export function gridPath(url: Located): string {
  return `${url.pathname}${url.search}`;
}

export function resumeId(url: Located): string | null {
  const path = gridPath(url);
  const active = browseView.lastGridPath === path ? browseView.activeId : null;
  return active ?? browseView.getGridAnchor(path)?.id ?? null;
}
