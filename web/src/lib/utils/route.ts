import { page } from '$app/state';

export function isEditorPath(pathname: string): boolean {
  return pathname.startsWith('/assets/');
}

export function onEditorRoute(): boolean {
  return isEditorPath(page.url.pathname);
}

export function currentPath(): string {
  return `${page.url.pathname}${page.url.search}`;
}

export function returnPath(): string | null {
  return page.url.searchParams.get('from');
}
