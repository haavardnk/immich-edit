import { ui } from '#lib/stores/ui.svelte.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { FIT_VIEW, type View, type ViewportMetrics } from '#lib/utils/imageViewport.ts';
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
  it('shows the Immich preview and a busy pane until the render loads', () => {
    component = mount(Host, {
      target,
      props: {
        component: LoupePane,
        props: { assetId: 'a', alt: 'a.jpg', view: FIT_VIEW, onView: () => {} }
      }
    });
    flushSync();
    const pane = target.querySelector<HTMLElement>('[role="button"]');
    expect(pane?.getAttribute('aria-busy')).toBe('true');
    expect(immichPreview()?.getAttribute('src')).toBe('/api/assets/a/thumb?size=preview');

    target.querySelector('img[alt="a.jpg"]')?.dispatchEvent(new Event('load'));
    flushSync();

    expect(pane?.getAttribute('aria-busy')).toBe('false');
    expect(immichPreview()).toBeNull();
  });

  it('keeps the loaded render when a render at a new size fails', () => {
    loadedPane(FIT_VIEW, () => {});
    const first = renderImage()?.getAttribute('src');
    try {
      ui.clipWarn = true;
      flushSync();
      expect(renderImage()?.getAttribute('src')).not.toBe(first);

      renderImage()?.dispatchEvent(new Event('error'));
      flushSync();

      expect(renderImage()?.getAttribute('src')).toBe(first);
      expect(immichPreview()).toBeNull();
    } finally {
      ui.clipWarn = false;
    }
  });

  it('leaves the Immich preview showing when the first render fails', () => {
    component = mount(Host, {
      target,
      props: {
        component: LoupePane,
        props: { assetId: 'a', alt: 'a.jpg', view: FIT_VIEW, onView: () => {} }
      }
    });
    flushSync();
    renderImage()?.dispatchEvent(new Event('error'));
    flushSync();

    expect(renderImage()).toBeNull();
    expect(immichPreview()?.getAttribute('src')).toBe('/api/assets/a/thumb?size=preview');
  });

  it('zooms the Immich preview with the pane before the render loads', () => {
    component = mount(Host, {
      target,
      props: {
        component: LoupePane,
        props: { assetId: 'a', alt: 'a.jpg', view: ZOOMED, onView: () => {}, sourceLong: 4000 }
      }
    });
    flushSync();
    const preview = immichPreview();
    if (!preview) throw new Error('no Immich preview');
    Object.defineProperty(preview, 'naturalWidth', { value: 1440 });
    Object.defineProperty(preview, 'naturalHeight', { value: 720 });
    preview.dispatchEvent(new Event('load'));
    flushSync();

    const style = preview.getAttribute('style') ?? '';
    expect(style).toContain('scale(');
    expect(target.querySelector('img[alt="a.jpg"]')?.getAttribute('style')).toBe(style);
  });

  it('flags a slow render after a short delay', () => {
    vi.useFakeTimers();
    try {
      component = mount(Host, {
        target,
        props: {
          component: LoupePane,
          props: { assetId: 'a', alt: 'a.jpg', view: FIT_VIEW, onView: () => {} }
        }
      });
      flushSync();
      const status = target.querySelector('[role="status"]');
      expect(status?.textContent?.trim()).toBe('');

      vi.advanceTimersByTime(399);
      flushSync();
      expect(status?.textContent?.trim()).toBe('');

      vi.advanceTimersByTime(1);
      flushSync();
      expect(status?.textContent?.trim()).toBe('Immich preview · rendering');

      target.querySelector('img[alt="a.jpg"]')?.dispatchEvent(new Event('load'));
      flushSync();
      expect(status?.textContent?.trim()).toBe('');
    } finally {
      vi.useRealTimers();
    }
  });

  it('never flags a render that loads quickly', () => {
    vi.useFakeTimers();
    try {
      component = mount(Host, {
        target,
        props: {
          component: LoupePane,
          props: { assetId: 'a', alt: 'a.jpg', view: FIT_VIEW, onView: () => {} }
        }
      });
      flushSync();
      vi.advanceTimersByTime(200);
      target.querySelector('img[alt="a.jpg"]')?.dispatchEvent(new Event('load'));
      flushSync();
      vi.advanceTimersByTime(1000);
      flushSync();
      expect(target.querySelector('[data-testid="preview-status"]')).toBeNull();
    } finally {
      vi.useRealTimers();
    }
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
          view: FIT_VIEW,
          onView,
          onMetrics: (metrics: ViewportMetrics) => (fitZoom = metrics.fitZoom)
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

function renderImage(): HTMLImageElement | null {
  return target.querySelector<HTMLImageElement>('img[alt="a.jpg"]');
}

function immichPreview(): HTMLImageElement | null {
  return target.querySelector<HTMLImageElement>('[data-testid="loupe-immich-preview"]');
}

function loadedPane(view: View, onView: PaneProps['onView'], sourceLong?: number): HTMLElement {
  component = mount(Host, {
    target,
    props: {
      component: LoupePane,
      props: { assetId: 'a', alt: 'a.jpg', view, onView, sourceLong }
    }
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

const ZOOMED: View = { zoom: 400, cx: 0.5, cy: 0.5 };

describe('LoupePane wheel', () => {
  it('zooms out below fit, centred, on a modifier wheel', () => {
    const onView = vi.fn();
    const pane = loadedPane(FIT_VIEW, onView, 4000);
    pane.dispatchEvent(
      new WheelEvent('wheel', { deltaY: 100, ctrlKey: true, clientX: 700, clientY: 400 })
    );
    const [view] = onView.mock.lastCall ?? [];
    expect(view.zoom).toBeCloseTo(25 / 1.1, 9);
    expect(view.cx).toBe(0.5);
    expect(view.cy).toBe(0.5);
  });

  it('pans a zoomed pane within the photo', () => {
    const onView = vi.fn();
    const pane = loadedPane({ zoom: 100, cx: 0.5, cy: 0.5 }, onView, 4000);
    pane.dispatchEvent(new WheelEvent('wheel', { deltaX: 4000, deltaY: -400 }));
    expect(onView).toHaveBeenLastCalledWith({ zoom: 100, cx: 0.875, cy: 0.3 }, false);
  });

  it('starts a pan from the shown view', () => {
    const onView = vi.fn();
    const pane = loadedPane({ zoom: 100, cx: 0.99, cy: 0.5 }, onView, 4000);
    pane.dispatchEvent(new WheelEvent('wheel', { deltaX: -400 }));
    const [view, solo] = onView.mock.lastCall ?? [];
    expect(solo).toBe(false);
    expect(view.cx).toBeCloseTo(0.775, 9);
    expect(view.cy).toBe(0.5);
  });
});

describe('LoupePane presses', () => {
  it('zooms in on a click', () => {
    const onView = vi.fn();
    const pane = loadedPane(FIT_VIEW, onView);
    pointer(pane, 'pointerdown');
    pointer(pane, 'pointerup');
    expect(onView).toHaveBeenCalledOnce();
    expect(onView.mock.lastCall?.[0].zoom).not.toBeNull();
  });

  it('keeps the clicked point under the pointer', () => {
    const onView = vi.fn();
    const pane = loadedPane(FIT_VIEW, onView, 4000);
    pointer(pane, 'pointerdown');
    pointer(pane, 'pointerup');
    const [view, solo] = onView.mock.lastCall ?? [];
    expect(solo).toBe(false);
    expect(view.zoom).toBe(100);
    expect(view.cx).toBeCloseTo(0.575, 9);
    expect(view.cy).toBeCloseTo(0.65, 9);
  });

  it('zooms out on a click without travel', () => {
    const onView = vi.fn();
    const pane = loadedPane(ZOOMED, onView);
    pointer(pane, 'pointerdown');
    pointer(pane, 'pointerup');
    expect(onView).toHaveBeenLastCalledWith(FIT_VIEW, false);
  });

  it('pans past the threshold without zooming out', () => {
    const onView = vi.fn();
    const pane = loadedPane(ZOOMED, onView);
    pointer(pane, 'pointerdown');
    pointer(pane, 'pointermove', 1, 160);
    pointer(pane, 'pointerup', 1, 160);
    expect(onView).toHaveBeenCalledOnce();
    expect(onView.mock.lastCall?.[0]).not.toEqual(FIT_VIEW);
  });

  it('only focuses an unfocused pane on its first click', () => {
    const onView = vi.fn();
    const onFocus = vi.fn();
    component = mount(Host, {
      target,
      props: {
        component: LoupePane,
        props: { assetId: 'a', alt: 'a.jpg', view: FIT_VIEW, onView, onFocus, showFocus: true }
      }
    });
    flushSync();
    const pane = target.querySelector<HTMLElement>('[role="button"]');
    if (!pane) throw new Error('no pane');
    pointer(pane, 'pointerdown');
    pointer(pane, 'pointerup');
    expect(onFocus).toHaveBeenCalledOnce();
    expect(onView).not.toHaveBeenCalled();
  });

  it('does not zoom in after a press that travelled', () => {
    const onView = vi.fn();
    const pane = loadedPane(FIT_VIEW, onView);
    pointer(pane, 'pointerdown');
    pointer(pane, 'pointermove', 1, 160);
    pointer(pane, 'pointerup', 1, 160);
    expect(onView).not.toHaveBeenCalled();
  });

  it('does not treat a cancelled press as a click', () => {
    const onView = vi.fn();
    const pane = loadedPane(FIT_VIEW, onView);
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
