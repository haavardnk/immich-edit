import { tokenMs } from '#lib/motion/tokens.ts';
import { cubicOut } from 'svelte/easing';
import { prefersReducedMotion } from 'svelte/motion';
import { slide, type TransitionConfig } from 'svelte/transition';

export function reveal(node: Element, { axis = 'y' }: { axis?: 'x' | 'y' } = {}): TransitionConfig {
  return slide(node, {
    axis,
    duration: prefersReducedMotion.current ? 0 : tokenMs('--transition-duration-base'),
    easing: cubicOut
  });
}
