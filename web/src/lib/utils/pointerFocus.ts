export const CONTROL =
  'button, a[href], summary, input[type="checkbox"], input[type="radio"], [role="button"], [role="link"], [role="tab"], [role="radio"], [role="checkbox"], [role="switch"], [role="option"], [role="menuitem"], [role="combobox"]';
const OWNS_ARROWS =
  'input[type="checkbox"], input[type="radio"], [role="tab"], [role="radio"], [role="checkbox"], [role="switch"]';
const KEEPS_FOCUS = '[role="dialog"], [role="alertdialog"], [role="menu"], [role="listbox"]';
const POINTER_FOCUS_MS = 100;

let pointerAt = -Infinity;
let pointerFocused: EventTarget | null = null;

function pointerControl(target: EventTarget | null): HTMLElement | null {
  if (!(target instanceof Element)) return null;
  const el = target.closest<HTMLElement>(CONTROL);
  if (!el || el.closest(KEEPS_FOCUS)) return null;
  return el;
}

export function isPointerFocused(target: EventTarget | null): boolean {
  return target !== null && target === pointerFocused && target === document.activeElement;
}

function notePointer(e: PointerEvent): void {
  pointerAt = e.timeStamp;
}

function noteFocus(e: FocusEvent): void {
  pointerFocused = e.timeStamp - pointerAt < POINTER_FOCUS_MS ? e.target : null;
}

function releasePointerFocus(e: MouseEvent): void {
  if (e.detail === 0) return;
  const el = pointerControl(e.target);
  if (el && isPointerFocused(el) && el.matches(OWNS_ARROWS)) el.blur();
}

function guardControlKeys(e: KeyboardEvent): void {
  if (e.key !== 'Enter' && e.key !== ' ') return;
  const el = pointerControl(e.target);
  if (!el || !isPointerFocused(el)) return;
  window.addEventListener(
    'keydown',
    (last) => {
      if (last === e) e.preventDefault();
    },
    { once: true }
  );
}

export function installFocusPolicy(): () => void {
  document.addEventListener('pointerdown', notePointer, { capture: true });
  document.addEventListener('focusin', noteFocus, { capture: true });
  document.addEventListener('click', releasePointerFocus);
  window.addEventListener('keydown', guardControlKeys, { capture: true });
  return () => {
    document.removeEventListener('pointerdown', notePointer, { capture: true });
    document.removeEventListener('focusin', noteFocus, { capture: true });
    document.removeEventListener('click', releasePointerFocus);
    window.removeEventListener('keydown', guardControlKeys, { capture: true });
  };
}
