import { flashElement } from '#lib/utils/flash.ts';

const WAIT_MS = 3000;

function find(root: HTMLElement, anchor: string): HTMLElement | null {
  return root.querySelector<HTMLElement>(`[data-setting-anchor="${CSS.escape(anchor)}"]`);
}

export function revealAnchor(root: HTMLElement, anchor: string): () => void {
  const now = find(root, anchor);
  if (now) {
    flashElement(now);
    return () => {};
  }
  const observer = new MutationObserver(() => {
    const found = find(root, anchor);
    if (!found) return;
    stop();
    flashElement(found);
  });
  const timer = setTimeout(() => stop(), WAIT_MS);
  function stop(): void {
    observer.disconnect();
    clearTimeout(timer);
  }
  observer.observe(root, { childList: true, subtree: true });
  return stop;
}
