import { describe, expect, it } from 'vitest';
import { lockPageZoom } from './pageZoom';

function pinch(type: string): boolean {
  const event = new Event(type, { bubbles: true, cancelable: true });
  document.body.dispatchEvent(event);
  return event.defaultPrevented;
}

describe('lockPageZoom', () => {
  it('cancels the WebKit pinch gesture until released', () => {
    const release = lockPageZoom();
    expect(pinch('gesturestart')).toBe(true);
    expect(pinch('gesturechange')).toBe(true);
    release();
    expect(pinch('gesturestart')).toBe(false);
  });
});
