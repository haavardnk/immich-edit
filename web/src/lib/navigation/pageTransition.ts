import type { OnNavigate } from '$app/navigation';
import { COLLECTIONS } from '#lib/components/library/collections.ts';
import { NAV_TABS } from '#lib/components/shell/touch/navTabs.ts';
import { pageDirection, pageMotion } from '#lib/navigation/pageDirection.ts';
import { prefersReducedMotion } from 'svelte/motion';

const TAB_ROOTS: ReadonlySet<string> = new Set(NAV_TABS.map((tab) => tab.href));
const SIDEBAR_ROOTS: ReadonlySet<string> = new Set([
  ...TAB_ROOTS,
  ...Object.values(COLLECTIONS).map((collection) => collection.href),
  '/settings'
]);

let generation = 0;

function ignore(): void {}

export function animatePageChange(
  navigation: OnNavigate,
  compact: boolean
): Promise<void> | undefined {
  const { from, to } = navigation;
  if (!from || !to) return;
  if (typeof document.startViewTransition !== 'function' || prefersReducedMotion.current) return;
  const roots = compact ? TAB_ROOTS : SIDEBAR_ROOTS;
  const motion = pageMotion(pageDirection(from.url, to.url, roots), compact);
  if (!motion) return;
  const root = document.documentElement;
  const mine = ++generation;
  root.dataset.pageMotion = motion;
  return new Promise((resolve) => {
    const transition = document.startViewTransition(async () => {
      resolve();
      await navigation.complete;
    });
    transition.ready.catch(ignore);
    transition.updateCallbackDone.catch(ignore);
    void transition.finished.catch(ignore).finally(() => {
      if (mine === generation) delete root.dataset.pageMotion;
    });
  });
}
