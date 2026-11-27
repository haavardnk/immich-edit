import { page } from '$app/state';
import { VIEWER_PATH } from '#lib/viewer/protocol.ts';

export function isEditorPath(pathname: string): boolean {
  return pathname.startsWith('/assets/');
}

export function onEditorRoute(): boolean {
  return isEditorPath(page.url.pathname);
}

export function onMapRoute(): boolean {
  return page.url.pathname === '/map';
}

export function isViewerPath(pathname: string): boolean {
  return pathname === VIEWER_PATH;
}

export function onViewerRoute(): boolean {
  return isViewerPath(page.url.pathname);
}

export function liveUrl(): typeof page.url {
  return page.shallow?.url ?? page.url;
}

export function currentPath(): string {
  const url = liveUrl();
  return `${url.pathname}${url.search}`;
}

export function returnPath(): string | null {
  return page.url.searchParams.get('from');
}
