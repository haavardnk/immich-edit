// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { centerWithin } from '#lib/utils/scrollReveal.ts';

function boxed(top: number, height: number): HTMLElement {
  const element = document.createElement('div');
  element.getBoundingClientRect = () => new DOMRect(0, top, 100, height);
  return element;
}

describe('centerWithin', () => {
  it('leaves a fully visible row alone', () => {
    const scroller = boxed(0, 400);
    scroller.scrollTop = 50;
    centerWithin(boxed(100, 40), scroller);
    expect(scroller.scrollTop).toBe(50);
  });

  it('centres a row below the view', () => {
    const scroller = boxed(0, 400);
    centerWithin(boxed(600, 40), scroller);
    expect(scroller.scrollTop).toBe(420);
  });
});
