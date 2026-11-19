import { flushSync } from 'svelte';
import { prefersReducedMotion } from 'svelte/motion';

let generation = 0;

function ignore(): void {}

export function swapView(kind: string, update: () => void): void {
  if (
    typeof document === 'undefined' ||
    typeof document.startViewTransition !== 'function' ||
    prefersReducedMotion.current
  ) {
    update();
    return;
  }
  const root = document.documentElement;
  const mine = ++generation;
  root.dataset.viewSwap = kind;
  const transition = document.startViewTransition(() => flushSync(update));
  transition.ready.catch(ignore);
  transition.updateCallbackDone.catch(ignore);
  void transition.finished.catch(ignore).finally(() => {
    if (mine === generation) delete root.dataset.viewSwap;
  });
}
