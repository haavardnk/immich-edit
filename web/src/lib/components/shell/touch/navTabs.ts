import { mdiBookshelf, mdiImageAlbum, mdiImageMultipleOutline, mdiMagnify } from '@mdi/js';

type NavTabId = 'photos' | 'albums' | 'search' | 'library';

type NavTab = {
  id: NavTabId;
  href: string;
  label: string;
  icon: string;
};

export const NAV_TABS: readonly NavTab[] = [
  { id: 'photos', href: '/photos', label: 'Photos', icon: mdiImageMultipleOutline },
  { id: 'albums', href: '/albums', label: 'Albums', icon: mdiImageAlbum },
  { id: 'search', href: '/search', label: 'Search', icon: mdiMagnify },
  { id: 'library', href: '/library', label: 'Library', icon: mdiBookshelf }
];

const TAB_ROOTS: readonly { tab: NavTabId; roots: readonly string[] }[] = [
  { tab: 'photos', roots: ['/photos'] },
  { tab: 'albums', roots: ['/albums'] },
  { tab: 'search', roots: ['/search'] },
  { tab: 'library', roots: ['/library', '/favorites', '/people', '/tags', '/folders', '/edited'] }
];

function within(pathname: string, root: string): boolean {
  return pathname === root || pathname.startsWith(`${root}/`);
}

export function activeTab(pathname: string): NavTabId | null {
  return TAB_ROOTS.find(({ roots }) => roots.some((root) => within(pathname, root)))?.tab ?? null;
}
