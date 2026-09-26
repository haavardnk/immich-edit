import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import LoupePane from './LoupePane.svelte';
import ComponentHost from '$lib/testing/ComponentHost.svelte';
import { CENTERED } from '$lib/stores/compare.svelte';

type PaneProps = ComponentProps<typeof LoupePane>;

const Host: Component<{ component: Component<PaneProps>; props: PaneProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

beforeEach(() => {
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
});

describe('LoupePane', () => {
  it('shows the thumbnail and a busy pane until the preview loads', () => {
    component = mount(Host, {
      target,
      props: {
        component: LoupePane,
        props: { assetId: 'a', alt: 'a.jpg', view: CENTERED, onView: () => {} }
      }
    });
    flushSync();
    const pane = target.querySelector<HTMLElement>('[role="button"]');
    expect(pane?.getAttribute('aria-busy')).toBe('true');
    expect(target.querySelector('[data-testid="loupe-underlay"]')).not.toBeNull();

    target.querySelector('img[alt="a.jpg"]')?.dispatchEvent(new Event('load'));
    flushSync();

    expect(pane?.getAttribute('aria-busy')).toBe('false');
    expect(target.querySelector('[data-testid="loupe-underlay"]')).toBeNull();
  });

  it('zooms in at the pointer on a modifier wheel', () => {
    const onView = vi.fn();
    let fitZoom = 0;
    component = mount(Host, {
      target,
      props: {
        component: LoupePane,
        props: {
          assetId: 'a',
          alt: 'a.jpg',
          view: CENTERED,
          onView,
          onFitZoom: (value: number) => (fitZoom = value)
        }
      }
    });
    flushSync();
    const image = target.querySelector<HTMLImageElement>('img[alt="a.jpg"]');
    if (!image) throw new Error('no preview image');
    Object.defineProperty(image, 'naturalWidth', { value: 2000 });
    Object.defineProperty(image, 'naturalHeight', { value: 1000 });
    image.dispatchEvent(new Event('load'));
    flushSync();

    target
      .querySelector('[role="button"]')
      ?.dispatchEvent(
        new WheelEvent('wheel', { deltaY: -100, ctrlKey: true, clientX: 700, clientY: 400 })
      );

    const [next] = onView.mock.lastCall ?? [];
    expect(next.zoom).toBeGreaterThan(fitZoom);
    expect(next.cx).toBeGreaterThan(0.5);
  });

  describe('touch', () => {
    function mountPane(props: Partial<PaneProps> = {}): {
      pane: HTMLElement;
      fitZoom: () => number;
    } {
      let fit = 0;
      component = mount(Host, {
        target,
        props: {
          component: LoupePane,
          props: {
            assetId: 'a',
            alt: 'a.jpg',
            view: CENTERED,
            onView: () => {},
            onFitZoom: (value: number) => (fit = value),
            sourceLong: 6000,
            ...props
          }
        }
      });
      flushSync();
      const pane = target.querySelector<HTMLElement>('[role="button"]');
      const image = target.querySelector<HTMLImageElement>('img[alt="a.jpg"]');
      if (!pane || !image) throw new Error('no pane');
      Object.defineProperty(pane, 'clientWidth', { value: 1000 });
      Object.defineProperty(pane, 'clientHeight', { value: 800 });
      pane.getBoundingClientRect = () => new DOMRect(0, 0, 1000, 800);
      pane.setPointerCapture = () => {};
      Object.defineProperty(image, 'naturalWidth', { value: 2000 });
      Object.defineProperty(image, 'naturalHeight', { value: 1000 });
      image.dispatchEvent(new Event('load'));
      flushSync();
      return { pane, fitZoom: () => fit };
    }

    function send(
      pane: HTMLElement,
      type: string,
      id: number,
      x: number,
      y: number,
      pointerType = 'touch'
    ): void {
      const event = new MouseEvent(type, {
        bubbles: true,
        cancelable: true,
        clientX: x,
        clientY: y
      });
      Object.defineProperties(event, {
        pointerId: { value: id },
        pointerType: { value: pointerType }
      });
      pane.dispatchEvent(event);
      flushSync();
    }

    it.each<[string, number, 1 | -1]>([
      ['left', -200, 1],
      ['right', 200, -1]
    ])('a %s swipe while fitted moves to the neighbour', (_name, dx, delta) => {
      const onSwipe = vi.fn();
      const { pane } = mountPane({ onSwipe });
      send(pane, 'pointerdown', 1, 500, 400);
      send(pane, 'pointermove', 1, 500 + dx, 410);
      send(pane, 'pointerup', 1, 500 + dx, 410);
      expect(onSwipe).toHaveBeenCalledWith(delta);
    });

    it('a mouse drag never swipes', () => {
      const onSwipe = vi.fn();
      const { pane } = mountPane({ onSwipe });
      send(pane, 'pointerdown', 1, 500, 400, 'mouse');
      send(pane, 'pointerup', 1, 300, 400, 'mouse');
      expect(onSwipe).not.toHaveBeenCalled();
    });

    it('one tap does nothing and a double tap zooms in', () => {
      const onView = vi.fn();
      const { pane, fitZoom } = mountPane({ onView });
      send(pane, 'pointerdown', 1, 700, 400);
      send(pane, 'pointerup', 1, 700, 400);
      expect(onView).not.toHaveBeenCalled();
      send(pane, 'pointerdown', 2, 702, 401);
      send(pane, 'pointerup', 2, 702, 401);
      const [next] = onView.mock.lastCall ?? [];
      expect(next.zoom).toBeGreaterThan(fitZoom());
      expect(next.cx).toBeGreaterThan(0.5);
    });

    it('spreading two fingers zooms in about their midpoint', () => {
      const onView = vi.fn();
      const { pane, fitZoom } = mountPane({ onView });
      send(pane, 'pointerdown', 1, 650, 400);
      send(pane, 'pointerdown', 2, 750, 400);
      send(pane, 'pointermove', 1, 550, 400);
      send(pane, 'pointermove', 2, 850, 400);
      const [next] = onView.mock.lastCall ?? [];
      expect(next.zoom).toBeGreaterThan(fitZoom());
      expect(next.cx).toBeGreaterThan(0.5);
    });
  });
});
