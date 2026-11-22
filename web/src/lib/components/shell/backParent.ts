import type { ReadonlyURL } from '$app/state';
import { folderHref, parentFolder } from '#lib/browse/folders.ts';

const HOME = '/photos';

const PARENTS: readonly { within: string; parent: string }[] = [
  { within: '/people', parent: '/people' },
  { within: '/tags', parent: '/tags' },
  { within: '/albums', parent: '/albums' },
  { within: '/settings', parent: '/settings' }
];

const LIBRARY_PAGES: ReadonlySet<string> = new Set([
  '/favorites',
  '/map',
  '/edited',
  '/people',
  '/tags'
]);

function folderParent(url: ReadonlyURL): string {
  const path = url.searchParams.get('path');
  return path ? folderHref(parentFolder(path)) : '/library';
}

export function backParent(url: ReadonlyURL): string {
  const { pathname } = url;
  if (pathname === '/folders') return folderParent(url);
  if (LIBRARY_PAGES.has(pathname)) return '/library';
  const nested = PARENTS.find(({ within }) => pathname.startsWith(`${within}/`));
  return nested?.parent ?? HOME;
}

const LEAVE_PAGE: ReadonlySet<string> = new Set(['/map']);

export function backSkips(current: string, previous: string): boolean {
  return LEAVE_PAGE.has(current) && previous === current;
}
