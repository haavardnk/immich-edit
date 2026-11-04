import { clickGuard } from '#lib/gestures/clickGuard.ts';
import { afterEach, describe, expect, it, vi } from 'vitest';

let cleanups: (() => void)[] = [];

afterEach(() => {
  for (const cleanup of cleanups) cleanup();
  cleanups = [];
});

function guarded() {
  const element = document.createElement('div');
  const button = document.createElement('button');
  const clicked = vi.fn();
  button.addEventListener('click', clicked);
  element.append(button);
  document.body.append(element);
  const guard = clickGuard();
  const cleanup = guard.attachment(element);
  cleanups.push(() => {
    if (typeof cleanup === 'function') cleanup();
    element.remove();
  });
  return { guard, button, clicked };
}

function click(target: Element, detail = 1): void {
  target.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, detail }));
}

describe('clickGuard', () => {
  it('lets clicks through while disarmed', () => {
    const { button, clicked } = guarded();
    click(button);
    expect(clicked).toHaveBeenCalledTimes(1);
  });

  it('swallows the one click that follows arming', () => {
    const { guard, button, clicked } = guarded();
    guard.arm();
    click(button);
    expect(clicked).not.toHaveBeenCalled();
    click(button);
    expect(clicked).toHaveBeenCalledTimes(1);
  });

  it('disarms on the next press', () => {
    const { guard, button, clicked } = guarded();
    guard.arm();
    button.dispatchEvent(new Event('pointerdown', { bubbles: true }));
    click(button);
    expect(clicked).toHaveBeenCalledTimes(1);
  });

  it('never swallows keyboard clicks', () => {
    const { guard, button, clicked } = guarded();
    guard.arm();
    click(button, 0);
    expect(clicked).toHaveBeenCalledTimes(1);
  });

  it('stops listeners on the guarded element itself', () => {
    const element = document.createElement('button');
    document.body.append(element);
    const guard = clickGuard();
    const cleanup = guard.attachment(element);
    const clicked = vi.fn();
    element.addEventListener('click', clicked);
    cleanups.push(() => {
      if (typeof cleanup === 'function') cleanup();
      element.remove();
    });
    guard.arm();
    click(element);
    expect(clicked).not.toHaveBeenCalled();
    click(element);
    expect(clicked).toHaveBeenCalledTimes(1);
  });

  it('marks the swallowed click as handled', () => {
    const { guard, button } = guarded();
    guard.arm();
    const event = new MouseEvent('click', { bubbles: true, cancelable: true, detail: 1 });
    button.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
  });

  it('cancels the lifting touch so no click lands on what opened under it', () => {
    const { guard, button, clicked } = guarded();
    guard.arm();
    const lift = new Event('touchend', { bubbles: true, cancelable: true });
    button.dispatchEvent(lift);
    expect(lift.defaultPrevented).toBe(true);
    click(button);
    expect(clicked).toHaveBeenCalledTimes(1);
  });

  it('falls back to swallowing the click when the lift cannot be cancelled', () => {
    const { guard, button, clicked } = guarded();
    guard.arm();
    button.dispatchEvent(new Event('touchend', { bubbles: true, cancelable: false }));
    click(button);
    expect(clicked).not.toHaveBeenCalled();
  });

  it('leaves an unarmed lift alone', () => {
    const { button } = guarded();
    const lift = new Event('touchend', { bubbles: true, cancelable: true });
    button.dispatchEvent(lift);
    expect(lift.defaultPrevented).toBe(false);
  });
});
