const SCROLLING = /(auto|scroll)/;

export function scrollParent(element: HTMLElement): HTMLElement | null {
  let node = element.parentElement;
  while (node && !SCROLLING.test(getComputedStyle(node).overflowY)) node = node.parentElement;
  return node;
}

export function revealWithin(element: HTMLElement, scroller: HTMLElement, margin = 0): void {
  const box = element.getBoundingClientRect();
  const view = scroller.getBoundingClientRect();
  if (box.top - margin < view.top) scroller.scrollTop -= view.top - box.top + margin;
  else if (box.bottom + margin > view.bottom)
    scroller.scrollTop += box.bottom - view.bottom + margin;
}

export function revealAcross(element: HTMLElement, scroller: HTMLElement, margin = 0): void {
  const box = element.getBoundingClientRect();
  const view = scroller.getBoundingClientRect();
  if (box.left - margin < view.left) scroller.scrollLeft -= view.left - box.left + margin;
  else if (box.right + margin > view.right) scroller.scrollLeft += box.right - view.right + margin;
}
