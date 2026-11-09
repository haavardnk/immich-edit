import type { Attachment } from 'svelte/attachments';

const positions = new Map<string, number>();
const SETTLE_MS = 1500;
const STOP_EVENTS = ['pointerdown', 'wheel', 'touchstart', 'keydown'] as const;

export function rememberScroll(key: string): Attachment<HTMLElement> {
  return (node) => {
    const target = positions.get(key) ?? 0;
    let pending = target > 0;
    const stop = (): void => {
      pending = false;
    };
    const place = (): void => {
      if (!pending) return;
      node.scrollTop = target;
      if (Math.abs(node.scrollTop - target) < 1) pending = false;
    };
    const save = (): void => {
      if (!pending) positions.set(key, node.scrollTop);
    };
    const observer = new ResizeObserver(place);
    for (const child of node.children) observer.observe(child);
    const timer = setTimeout(stop, SETTLE_MS);
    place();
    node.addEventListener('scroll', save, { passive: true });
    for (const type of STOP_EVENTS) node.addEventListener(type, stop, { passive: true });
    return () => {
      observer.disconnect();
      clearTimeout(timer);
      node.removeEventListener('scroll', save);
      for (const type of STOP_EVENTS) node.removeEventListener(type, stop);
    };
  };
}
