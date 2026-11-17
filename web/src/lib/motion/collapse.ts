import { tokenMs } from '#lib/motion/tokens.ts';
import { prefersReducedMotion } from 'svelte/motion';
import type { Attachment } from 'svelte/attachments';

const shown = new WeakMap<HTMLElement, boolean>();

export function collapseX(collapsed: boolean): Attachment<HTMLElement> {
  return (node) => {
    const was = shown.get(node);
    shown.set(node, !collapsed);
    if (was === undefined || was === !collapsed || prefersReducedMotion.current) {
      node.style.display = collapsed ? 'none' : '';
      return;
    }
    const width = `${node.offsetWidth || node.scrollWidth}px`;
    node.style.display = '';
    const full = collapsed ? width : `${node.offsetWidth}px`;
    const frames = collapsed
      ? [
          { width: full, overflow: 'hidden' },
          { width: '0px', overflow: 'hidden' }
        ]
      : [
          { width: '0px', overflow: 'hidden' },
          { width: full, overflow: 'hidden' }
        ];
    const animation = node.animate(frames, {
      duration: tokenMs('--transition-duration-base'),
      easing: 'cubic-bezier(0.33, 1, 0.68, 1)'
    });
    if (collapsed) animation.onfinish = () => (node.style.display = 'none');
  };
}
