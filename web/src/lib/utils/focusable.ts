const FOCUSABLE = 'a[href], button, input, select, textarea, [tabindex]';

export function focusables(root: ParentNode): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>(FOCUSABLE)];
}

export function canFocus(element: HTMLElement): boolean {
  return (
    element.isConnected &&
    element.tabIndex >= 0 &&
    !element.matches(':disabled') &&
    !element.closest('[inert]')
  );
}

export function firstFocusable(root: ParentNode): HTMLElement | null {
  return focusables(root).find(canFocus) ?? null;
}
