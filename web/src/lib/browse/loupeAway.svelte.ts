import { skipZoomIn } from '#lib/browse/loupeZoom.ts';
import type { PageTurn } from '#lib/navigation/pageTransition.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { routeKey } from '#lib/stores/navHistory.ts';
import type { NavigationType } from '$app/navigation';
import { page } from '$app/state';
import { flushSync } from 'svelte';

type Place = Pick<URL, 'pathname' | 'search'>;

export interface Leaving {
  type?: NavigationType;
  complete: Promise<void>;
}

interface Away {
  id: string;
  info: boolean;
  from: string;
  to: string;
}

const LISTED_MS = 5000;
const HOLD_MS = 800;

let away: Away | null = null;
let quiet: string | null = null;

function ignore(): void {}

function whenListed(id: string): Promise<boolean> {
  if (browsing.find(id)) return Promise.resolve(true);
  return new Promise((resolve) => {
    let done = false;
    let stop: (() => void) | null = null;
    const finish = (found: boolean): void => {
      if (done) return;
      done = true;
      clearTimeout(timer);
      stop?.();
      resolve(found);
    };
    const timer = setTimeout(() => finish(false), LISTED_MS);
    stop = $effect.root(() => {
      $effect(() => {
        if (browsing.find(id)) finish(true);
      });
    });
    if (done) stop();
  });
}

function reopen(id: string, info: boolean, still: boolean): void {
  if (still) skipZoomIn(id);
  flushSync(() => {
    browseView.openLoupe(id);
    browseView.setLoupeInfoOpen(info);
  });
}

function reopenWhenListed(left: Away): Promise<void> {
  let held = true;
  const hold = new Promise<void>((resolve) => setTimeout(resolve, HOLD_MS)).then(() => {
    held = false;
  });
  const shown = whenListed(left.id).then((listed) => {
    if (listed && !browseView.loupeId && routeKey(page.url) === left.from)
      reopen(left.id, left.info, held);
  });
  return Promise.race([hold, shown]);
}

export function leaveLoupeFor(id: string, from: Place, to: Place, leaving: Leaving): PageTurn {
  away = { id, info: browseView.loupeInfoOpen, from: routeKey(from), to: to.pathname };
  leaving.complete.then(
    () => {
      if (browseView.loupeId !== id) return;
      quiet = id;
      browseView.closeLoupe();
    },
    () => {
      away = null;
    }
  );
  return { direction: 'forward', whole: true };
}

export function returnToLoupe(
  from: Place | undefined,
  to: Place,
  leaving: Leaving
): PageTurn | null {
  const left = away;
  if (!left || from?.pathname !== left.to) return null;
  if (to.pathname === left.to) return null;
  away = null;
  if (routeKey(to) !== left.from || browseView.loupeId) return null;
  const ready = leaving.complete.then(() => reopenWhenListed(left), ignore);
  return { direction: 'back', whole: true, ready };
}

export function returnsToLoupe(previous: string): boolean {
  return away !== null && away.from === previous;
}

export function closedQuietly(id: string): boolean {
  if (quiet !== id) return false;
  quiet = null;
  return true;
}
