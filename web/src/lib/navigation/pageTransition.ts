import type { OnNavigate } from '$app/navigation';
import { COLLECTIONS } from '#lib/components/library/collections.ts';
import { NAV_TABS } from '#lib/components/shell/touch/navTabs.ts';
import { pageDirection, pageMotion, type PageDirection } from '#lib/navigation/pageDirection.ts';
import { prefersReducedMotion } from 'svelte/motion';

const TAB_ROOTS: ReadonlySet<string> = new Set(NAV_TABS.map((tab) => tab.href));
const SIDEBAR_ROOTS: ReadonlySet<string> = new Set([
  ...TAB_ROOTS,
  ...Object.values(COLLECTIONS).map((collection) => collection.href),
  '/settings'
]);

export interface PageTurn {
  direction: PageDirection;
  whole: boolean;
  ready?: Promise<void>;
}

interface PageChange {
  waiting: Promise<void>;
  release: () => void;
}

interface Unsettled {
  release: () => void;
  settled: Promise<void>;
}

let generation = 0;
let unsettled: Unsettled | null = null;

function ignore(): void {}

function startPageChange(
  navigation: OnNavigate,
  compact: boolean,
  turn: PageTurn | null
): PageChange | null {
  const { from, to } = navigation;
  if (!from || !to) return null;
  if (typeof document.startViewTransition !== 'function' || prefersReducedMotion.current) {
    return null;
  }
  const roots = compact ? TAB_ROOTS : SIDEBAR_ROOTS;
  const motion = pageMotion(turn?.direction ?? pageDirection(from.url, to.url, roots), compact);
  if (!motion) return null;
  const root = document.documentElement;
  const mine = ++generation;
  root.dataset.pageMotion = motion;
  if (turn?.whole) root.dataset.pageWhole = '';
  else delete root.dataset.pageWhole;
  let release = ignore;
  const waiting = new Promise<void>((resolve) => {
    const transition = document.startViewTransition(async () => {
      resolve();
      await navigation.complete;
      await turn?.ready;
    });
    release = () => transition.skipTransition();
    transition.ready.catch(ignore);
    transition.updateCallbackDone.catch(ignore);
    void transition.finished.catch(ignore).finally(() => {
      if (mine !== generation) return;
      delete root.dataset.pageMotion;
      delete root.dataset.pageWhole;
    });
  });
  return { waiting, release };
}

export function animatePageChange(
  navigation: OnNavigate,
  compact: boolean,
  turn: PageTurn | null = null
): Promise<void> | undefined {
  const before = unsettled;
  before?.release();
  const change = startPageChange(navigation, compact, turn);
  const record: Unsettled = {
    release: change?.release ?? ignore,
    settled: navigation.complete.then(ignore, ignore)
  };
  unsettled = record;
  void record.settled.then(() => {
    if (unsettled === record) unsettled = null;
  });
  if (!before) return change?.waiting;
  return before.settled.then(() => change?.waiting);
}
