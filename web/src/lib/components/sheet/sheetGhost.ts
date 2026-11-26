import { cssToken, tokenMs } from '#lib/motion/tokens.ts';
import { prefersReducedMotion } from 'svelte/motion';

const FRAME = '[data-sheet-frame]';
const PANEL = '[data-sheet-panel]';
const SCRIM = '[data-sheet-scrim]';

export interface SheetParts {
  frame: HTMLElement;
  scrim: HTMLElement | null;
}

export function sheetParts(content: Element): SheetParts | null {
  const frame = content.closest<HTMLElement>(FRAME);
  if (!frame) return null;
  const before = frame.previousElementSibling;
  const scrim = before instanceof HTMLElement && before.matches(SCRIM) ? before : null;
  return { frame, scrim };
}

function still(element: HTMLElement): HTMLElement {
  const copy = element.cloneNode(true) as HTMLElement;
  copy.inert = true;
  copy.setAttribute('aria-hidden', 'true');
  copy.removeAttribute('data-sheet-frame');
  copy.dataset.sheetGhost = '';
  copy.style.pointerEvents = 'none';
  for (const node of [copy, ...copy.querySelectorAll<HTMLElement>('*')]) {
    node.style.transition = 'none';
  }
  return copy;
}

export function leaveBehind({ frame, scrim }: SheetParts): void {
  if (prefersReducedMotion.current) return;
  const timing: KeyframeAnimationOptions = {
    duration: tokenMs('--transition-duration-base'),
    easing: cssToken('--ease-exit') || 'ease-in',
    fill: 'forwards'
  };
  const ghosts: HTMLElement[] = [];
  const motions: Animation[] = [];
  if (scrim) {
    const shade = still(scrim);
    shade.removeAttribute('data-leaving');
    ghosts.push(shade);
    document.body.append(shade);
    motions.push(shade.animate([{ opacity: 1 }, { opacity: 0 }], timing));
  }
  const copy = still(frame);
  ghosts.push(copy);
  document.body.append(copy);
  const panel = copy.querySelector<HTMLElement>(PANEL);
  if (panel) {
    motions.push(panel.animate([{ translate: '0 0' }, { translate: '0 100%' }], timing));
  }
  void Promise.allSettled(motions.map((motion) => motion.finished)).then(() => {
    for (const ghost of ghosts) ghost.remove();
  });
}
