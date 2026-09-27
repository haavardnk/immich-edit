import { CONTROL, isPointerFocused } from '$lib/utils/pointerFocus';

export function isTypingTarget(e: KeyboardEvent): boolean {
  const el = e.target as HTMLElement | null;
  if (!el) return false;
  const tag = el.tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
  return el.isContentEditable;
}

export function isRadioGroupTarget(e: KeyboardEvent): boolean {
  const el = e.target as HTMLElement | null;
  return !!el?.closest('[role="radiogroup"]') && !isPointerFocused(el);
}

export function yieldsToControl(e: KeyboardEvent): boolean {
  if (e.key !== 'Enter' && e.key !== ' ') return false;
  const el = e.target as HTMLElement | null;
  return !!el?.matches(CONTROL) && !isPointerFocused(el);
}
