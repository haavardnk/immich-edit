import { canFocus, focusables } from '#lib/utils/focusable.ts';

function isRendered(element: HTMLElement): boolean {
  return element.checkVisibility?.() ?? element.getClientRects().length > 0;
}

export function focusTargetsAround(region: Element): HTMLElement[] {
  const outside = focusables(region.ownerDocument).filter(
    (element) => !region.contains(element) && !element.contains(region)
  );
  const before = outside.filter(
    (element) => region.compareDocumentPosition(element) & Node.DOCUMENT_POSITION_PRECEDING
  );
  const after = outside.filter(
    (element) => region.compareDocumentPosition(element) & Node.DOCUMENT_POSITION_FOLLOWING
  );
  return [...before.reverse(), ...after];
}

export function focusFirst(targets: readonly HTMLElement[]): void {
  targets.find((element) => canFocus(element) && isRendered(element))?.focus();
}
