import { page } from '$app/state';
import { VIEWER_PATH } from '#lib/viewer/protocol.ts';

export function isEditorPath(pathname: string): boolean {
  return pathname.startsWith('/assets/');
}

export function onEditorRoute(): boolean {
  return isEditorPath(page.url.pathname);
}

export function isViewerPath(pathname: string): boolean {
  return pathname === VIEWER_PATH;
}

export function onViewerRoute(): boolean {
  return isViewerPath(page.url.pathname);
}

export function currentPath(): string {
  return `${page.url.pathname}${page.url.search}`;
}

export function returnPath(): string | null {
  return page.url.searchParams.get('from');
}
