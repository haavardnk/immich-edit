import { browseView } from '#lib/stores/browseView.svelte.ts';

type Located = Pick<URL, 'pathname' | 'search'>;

const CAMERA_PARAM = 'at';

export function gridPath(url: Located): string {
  const params = new URLSearchParams(url.search);
  if (!params.has(CAMERA_PARAM)) return `${url.pathname}${url.search}`;
  params.delete(CAMERA_PARAM);
  const rest = params.toString();
  return rest ? `${url.pathname}?${rest}` : url.pathname;
}

export function resumeId(url: Located): string | null {
  const path = gridPath(url);
  const active = browseView.lastGridPath === path ? browseView.activeId : null;
  return active ?? browseView.getGridAnchor(path)?.id ?? null;
}
