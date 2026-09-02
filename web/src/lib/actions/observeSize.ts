import type { Action } from 'svelte/action';

export const observeSize: Action<Element, () => void> = (node, measure) => {
  measure();
  const observer = new ResizeObserver(measure);
  observer.observe(node);
  return {
    destroy(): void {
      observer.disconnect();
    }
  };
};
