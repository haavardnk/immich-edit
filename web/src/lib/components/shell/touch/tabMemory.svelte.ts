import {
  activeTab,
  NAV_TABS,
  type NavTab,
  type NavTabId
} from '#lib/components/shell/touch/navTabs.ts';

type Located = Pick<URL, 'pathname' | 'search'>;

function isRoot(tab: NavTabId, pathname: string): boolean {
  return NAV_TABS.some((root) => root.id === tab && root.href === pathname);
}

class TabMemory {
  #last = $state<Partial<Record<NavTabId, string>>>({});
  #root = $state<Partial<Record<NavTabId, string>>>({});

  visit(url: Located): void {
    const tab = activeTab(url.pathname);
    if (!tab) return;
    const href = `${url.pathname}${url.search}`;
    this.#last[tab] = href;
    if (isRoot(tab, url.pathname)) this.#root[tab] = href;
  }

  href(tab: NavTab, current: Located): string {
    if (activeTab(current.pathname) !== tab.id) return this.#last[tab.id] ?? tab.href;
    if (current.pathname === tab.href) return `${current.pathname}${current.search}`;
    return this.#root[tab.id] ?? tab.href;
  }
}

export const tabMemory = new TabMemory();
