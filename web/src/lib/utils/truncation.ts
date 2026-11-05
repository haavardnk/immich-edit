const TARGETS = '[data-truncate]';

export function isTruncated(element: Element): boolean {
  const targets = element.matches(TARGETS)
    ? [element]
    : Array.from(element.querySelectorAll(TARGETS));
  return targets.some((target) => target.scrollWidth > target.clientWidth);
}
