import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { listenGridZoom } from './gridZoomInput';

let element: HTMLDivElement;
let stop = (): void => {};
const step = vi.fn();

function stamped<E extends Event>(event: E, time: number): E {
  Object.defineProperty(event, 'timeStamp', { value: time });
  element.dispatchEvent(event);
  return event;
}

function wheel(init: WheelEventInit, time = 0): WheelEvent {
  return stamped(new WheelEvent('wheel', { bubbles: true, cancelable: true, ...init }), time);
}

function gesture(type: string, scale: number, time = 0): Event {
  const event = Object.assign(new Event(type, { bubbles: true, cancelable: true }), {
    scale,
    clientX: 40,
    clientY: 50
  });
  return stamped(event, time);
}

function touches(type: string, count: number): void {
  const event = Object.assign(new Event(type, { bubbles: true }), {
    touches: { length: count }
  });
  element.dispatchEvent(event);
}

beforeEach(() => {
  step.mockReset();
  element = document.createElement('div');
  document.body.appendChild(element);
  stop = listenGridZoom(element, step);
});

afterEach(() => {
  stop();
  element.remove();
});

describe('grid zoom input', () => {
  it('steps on a Ctrl or Cmd wheel at the pointer and keeps the page from zooming', () => {
    const zoomIn = wheel({ deltaY: -100, ctrlKey: true, clientX: 10, clientY: 20 }, 0);
    const zoomOut = wheel({ deltaY: 100, metaKey: true, clientX: 11, clientY: 21 }, 200);

    expect(zoomIn.defaultPrevented).toBe(true);
    expect(zoomOut.defaultPrevented).toBe(true);
    expect(step.mock.calls).toEqual([
      [1, { x: 10, y: 20 }],
      [-1, { x: 11, y: 21 }]
    ]);
  });

  it('follows a Ctrl pinch stream event by event but steps Cmd momentum once per notch', () => {
    for (const time of [0, 16, 32]) wheel({ deltaY: -30, ctrlKey: true }, time);
    expect(step).toHaveBeenCalledTimes(3);

    step.mockReset();
    for (let time = 1000; time < 1300; time += 16) wheel({ deltaY: -30, metaKey: true }, time);
    expect(step).toHaveBeenCalledTimes(2);

    step.mockReset();
    for (let time = 2000; time < 2100; time += 10) wheel({ deltaY: -100, ctrlKey: true }, time);
    expect(step).toHaveBeenCalledTimes(1);
  });

  it('leaves a plain wheel to scroll the grid', () => {
    const event = wheel({ deltaY: -100 });

    expect(event.defaultPrevented).toBe(false);
    expect(step).not.toHaveBeenCalled();
  });

  it('steps on a Safari trackpad pinch and ignores its wheel echo', () => {
    expect(gesture('gesturestart', 1, 0).defaultPrevented).toBe(true);
    gesture('gesturechange', 1.3, 10);
    wheel({ deltaY: -100, ctrlKey: true }, 15);
    gesture('gesturechange', 2.1, 20);
    gesture('gestureend', 2.1, 30);

    expect(step.mock.calls).toEqual([
      [1, { x: 40, y: 50 }],
      [2, { x: 40, y: 50 }]
    ]);
  });

  it('leaves a Safari touch pinch to the touch recognizer', () => {
    touches('touchstart', 2);
    expect(gesture('gesturestart', 1).defaultPrevented).toBe(true);
    gesture('gesturechange', 2);
    gesture('gestureend', 2);
    touches('touchend', 0);

    expect(step).not.toHaveBeenCalled();
  });

  it('stops blocking the wheel when a gesture never ends', () => {
    gesture('gesturestart', 1, 0);
    wheel({ deltaY: -100, ctrlKey: true }, 100);
    expect(step).not.toHaveBeenCalled();

    wheel({ deltaY: -100, ctrlKey: true, clientX: 5, clientY: 6 }, 1000);
    expect(step.mock.calls).toEqual([[1, { x: 5, y: 6 }]]);
  });

  it('starts a fresh gesture after a touch gesture that never ended', () => {
    touches('touchstart', 2);
    gesture('gesturestart', 1, 0);
    touches('touchend', 0);
    gesture('gesturestart', 1, 50);
    gesture('gesturechange', 1.3, 60);

    expect(step.mock.calls).toEqual([[1, { x: 40, y: 50 }]]);
  });

  it('stops listening when released', () => {
    stop();
    wheel({ deltaY: -100, ctrlKey: true });

    expect(step).not.toHaveBeenCalled();
  });
});
