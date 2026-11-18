import { animated, timing } from '#lib/motion/photoGhost.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';

const ROOT = '[data-loupe-root]';
export const LOUPE_RELEASE_FALLBACK_MS = 2500;

export function releaseLoupe(): void {
  if (!browseView.loupeBehindEditor) return;
  const root = document.querySelector(ROOT);
  if (!root || !animated()) {
    browseView.releaseLoupeBehindEditor();
    return;
  }
  void root
    .animate(
      [{ opacity: 1 }, { opacity: 0 }],
      timing('--transition-duration-fast', '--ease-exit', 'forwards')
    )
    .finished.catch(() => undefined)
    .then(() => browseView.releaseLoupeBehindEditor());
}
