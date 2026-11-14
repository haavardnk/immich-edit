import { afterEach, describe, expect, it, vi } from 'vitest';
import { edgeScroller, edgeSpeed } from './edgeScroll';

describe('edgeSpeed', () => {
  const top = 100;
  const bottom = 700;
  const full = edgeSpeed(bottom, top, bottom);

  it('scrolls down at the bottom edge and up at the top edge at the same pace', () => {
    expect(full).toBeGreaterThan(0);
    expect(edgeSpeed(top, top, bottom)).toBe(-full);
  });

  it('caps the pace once the pointer leaves the scroller', () => {
    expect(edgeSpeed(bottom + 200, top, bottom)).toBe(full);
    expect(edgeSpeed(top - 60, top, bottom)).toBe(-full);
  });

  it('idles in the middle', () => {
    expect(edgeSpeed(400, top, bottom)).toBe(0);
  });

  it('speeds up the deeper the pointer goes into a band', () => {
    let entry = bottom;
    while (edgeSpeed(entry - 1, top, bottom) > 0) entry -= 1;
    const band = bottom - entry + 1;
    expect(band).toBeGreaterThan(20);
    expect(band).toBeLessThan(100);
    expect(edgeSpeed(entry, top, bottom)).toBeGreaterThan(0);
    expect(edgeSpeed(entry, top, bottom)).toBeLessThan(full / 4);
    expect(edgeSpeed(bottom - band / 2, top, bottom)).toBeCloseTo(full / 2, 5);
  });

  it('splits a short scroller between its two bands', () => {
    expect(edgeSpeed(130, 100, 160)).toBe(0);
    expect(edgeSpeed(100, 100, 160)).toBe(-full);
  });
});

describe('edgeScroller', () => {
  afterEach(() => vi.unstubAllGlobals());

  function frames(): () => void {
    const queue: FrameRequestCallback[] = [];
    vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => {
      queue.push(callback);
      return queue.length;
    });
    vi.stubGlobal('cancelAnimationFrame', () => queue.splice(0));
    return () => queue.splice(0).forEach((callback) => callback(0));
  }

  function scroller(): HTMLElement {
    return {
      scrollTop: 500,
      getBoundingClientRect: () => ({ top: 0, bottom: 600 })
    } as unknown as HTMLElement;
  }

  it('keeps scrolling while the pointer rests in a band and stops on release', () => {
    const run = frames();
    const element = scroller();
    const onScroll = vi.fn();
    const step = edgeSpeed(600, 0, 600);
    const edge = edgeScroller(() => element, onScroll);
    edge.track(600);
    run();
    run();
    expect(element.scrollTop).toBe(500 + 2 * step);
    expect(onScroll).toHaveBeenCalledTimes(2);
    edge.stop();
    run();
    expect(element.scrollTop).toBe(500 + 2 * step);
  });

  it('idles when the pointer leaves the band', () => {
    const run = frames();
    const element = scroller();
    const edge = edgeScroller(
      () => element,
      () => {}
    );
    edge.track(300);
    run();
    run();
    expect(element.scrollTop).toBe(500);
  });
});
