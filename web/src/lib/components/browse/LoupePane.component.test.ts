import { CENTERED, type PaneView } from '#lib/stores/compare.svelte.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import LoupePane from './LoupePane.svelte';

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
});

function loadedPane(view: PaneView, onView: PaneProps['onView']): HTMLElement {
  component = mount(Host, {
    target,
    props: { component: LoupePane, props: { assetId: 'a', alt: 'a.jpg', view, onView } }
  });
  flushSync();
  const image = target.querySelector<HTMLImageElement>('img[alt="a.jpg"]');
  if (!image) throw new Error('no preview image');
  Object.defineProperty(image, 'naturalWidth', { value: 2000 });
  Object.defineProperty(image, 'naturalHeight', { value: 1000 });
  image.dispatchEvent(new Event('load'));
  flushSync();
  const pane = target.querySelector<HTMLElement>('[role="button"]');
  if (!pane) throw new Error('no pane');
  return pane;
}

function pointer(pane: HTMLElement, type: string, pointerId = 1, clientX = 100): void {
  pane.dispatchEvent(new PointerEvent(type, { bubbles: true, pointerId, clientX, clientY: 100 }));
  flushSync();
}

const ZOOMED: PaneView = { zoom: 400, cx: 0.5, cy: 0.5 };

describe('LoupePane presses', () => {
  it('zooms in on a click', () => {
    const onView = vi.fn();
    const pane = loadedPane(CENTERED, onView);
    pointer(pane, 'pointerdown');
    pointer(pane, 'pointerup');
    expect(onView).toHaveBeenCalledOnce();
    expect(onView.mock.lastCall?.[0].zoom).not.toBeNull();
  });

  it('zooms out on a click without travel', () => {
    const onView = vi.fn();
    const pane = loadedPane(ZOOMED, onView);
    pointer(pane, 'pointerdown');
    pointer(pane, 'pointerup');
    expect(onView).toHaveBeenLastCalledWith(CENTERED, false);
  });

  it('pans past the threshold without zooming out', () => {
    const onView = vi.fn();
    const pane = loadedPane(ZOOMED, onView);
    pointer(pane, 'pointerdown');
    pointer(pane, 'pointermove', 1, 160);
    pointer(pane, 'pointerup', 1, 160);
    expect(onView).toHaveBeenCalledOnce();
    expect(onView.mock.lastCall?.[0]).not.toEqual(CENTERED);
  });

  it('does not treat a cancelled press as a click', () => {
    const onView = vi.fn();
    const pane = loadedPane(CENTERED, onView);
    pointer(pane, 'pointerdown');
    pointer(pane, 'pointercancel');
    pointer(pane, 'pointerup');
    expect(onView).not.toHaveBeenCalled();
  });

  it('does not treat a pan that lost capture as a click', () => {
    const onView = vi.fn();
    const pane = loadedPane(ZOOMED, onView);
    pointer(pane, 'pointerdown');
    pointer(pane, 'lostpointercapture');
    expect(pane.classList.contains('cursor-grabbing')).toBe(false);
    pointer(pane, 'pointerup');
    expect(onView).not.toHaveBeenCalled();
  });

  it('ignores ups and cancels from another pointer during a pan', () => {
    const onView = vi.fn();
    const pane = loadedPane(ZOOMED, onView);
    pointer(pane, 'pointerdown', 1);
    pointer(pane, 'pointerup', 2);
    pointer(pane, 'pointercancel', 2);
    expect(pane.classList.contains('cursor-grabbing')).toBe(true);
    expect(onView).not.toHaveBeenCalled();
    pointer(pane, 'pointermove', 1, 160);
    expect(onView).toHaveBeenCalledOnce();
    pointer(pane, 'pointerup', 1, 160);
    expect(onView).toHaveBeenCalledOnce();
    expect(pane.classList.contains('cursor-grabbing')).toBe(false);
  });
});
