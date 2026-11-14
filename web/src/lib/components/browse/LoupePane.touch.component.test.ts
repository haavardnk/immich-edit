import { LoupeStage } from '#lib/browse/loupeStage.ts';
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
let frames: FrameRequestCallback[] = [];

beforeEach(() => {
  frames = [];
  vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => {
    frames.push(callback);
    return frames.length;
  });
  vi.stubGlobal('cancelAnimationFrame', () => {});
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  ui.fullscreen = false;
  vi.unstubAllGlobals();
});

function runFrames(): void {
  const pending = frames;
  frames = [];
  for (const callback of pending) callback(0);
}

function mountPane(props: Partial<PaneProps> = {}) {
  let fitZoom = 0;
  component = mount(Host, {
    target,
    props: {
      component: LoupePane,
      props: {
        assetId: 'a',
        alt: 'a.jpg',
        view: FIT_VIEW,
        onView: () => {},
        onMetrics: (metrics: ViewportMetrics) => (fitZoom = metrics.fitZoom),
        sourceLong: 4000,
        ...props
      }
    }
  });
  flushSync();
  const pane = target.querySelector<HTMLElement>('[role="button"]');
  const image = target.querySelector<HTMLImageElement>('img[alt="a.jpg"]');
  if (!pane || !image) throw new Error('no pane');
  pane.getBoundingClientRect = () => new DOMRect(0, 0, 1000, 800);
  pane.setPointerCapture = () => {};
  pane.releasePointerCapture = () => {};
  pane.hasPointerCapture = () => false;
  Object.defineProperty(image, 'naturalWidth', { value: 2000 });
  Object.defineProperty(image, 'naturalHeight', { value: 1000 });
  image.dispatchEvent(new Event('load'));
  flushSync();
  return { pane, fitZoom: () => fitZoom };
}

function send(
  pane: HTMLElement,
  type: string,
  pointerId: number,
  clientX: number,
  clientY: number,
  pointerType = 'touch'
): void {
  pane.dispatchEvent(
    new PointerEvent(type, {
      bubbles: true,
      cancelable: true,
      pointerId,
      pointerType,
      clientX,
      clientY,
      buttons: type === 'pointerup' ? 0 : 1
    })
  );
  runFrames();
  flushSync();
}

function drag(pane: HTMLElement, dx: number, dy: number, pointerType = 'touch'): void {
  send(pane, 'pointerdown', 1, 500, 400, pointerType);
  send(pane, 'pointermove', 1, 500 + dx / 2, 400 + dy / 2, pointerType);
  send(pane, 'pointermove', 1, 500 + dx, 400 + dy, pointerType);
  send(pane, 'pointerup', 1, 500 + dx, 400 + dy, pointerType);
}

function stageSpy() {
  const step = vi.fn();
  const dismiss = vi.fn();
  const openInfo = vi.fn();
  const track = document.createElement('div');
  const stage = new LoupeStage({
    track: () => track,
    scrim: () => null,
    canStep: () => true,
    hasNeighbour: () => true,
    current: () => 'a',
    step,
    dismiss,
    openInfo,
    reducedMotion: () => true
  });
  return { stage, step, dismiss, openInfo };
}

const ZOOMED: View = { zoom: 400, cx: 0.5, cy: 0.5 };

describe('LoupePane touch gestures', () => {
  it.each<[string, number, 1 | -1]>([
    ['left', -300, 1],
    ['right', 300, -1]
  ])('a %s swipe at fit steps to the neighbour', (_name, dx, delta) => {
    const { stage, step } = stageSpy();
    const { pane } = mountPane({ stage });
    drag(pane, dx, 10);
    expect(step).toHaveBeenCalledExactlyOnceWith(delta);
  });

  it('a mouse drag never swipes', () => {
    const { stage, step } = stageSpy();
    const onView = vi.fn();
    const { pane } = mountPane({ stage, onView });
    drag(pane, -300, 0, 'mouse');
    expect(step).not.toHaveBeenCalled();
    expect(onView).not.toHaveBeenCalled();
  });

  it('a swipe in a pane without a stage does nothing', () => {
    const onView = vi.fn();
    const { pane } = mountPane({ onView });
    drag(pane, -300, 0);
    expect(onView).not.toHaveBeenCalled();
  });

  it('one finger pans a zoomed photo instead of stepping', () => {
    const { stage, step } = stageSpy();
    const onView = vi.fn();
    const { pane } = mountPane({ stage, onView, view: ZOOMED });
    drag(pane, -300, 0);
    expect(step).not.toHaveBeenCalled();
    const [next] = onView.mock.lastCall ?? [];
    expect(next.zoom).toBe(400);
    expect(next.cx).toBeGreaterThan(0.5);
  });

  it('a swipe down dismisses and a swipe up opens the info', () => {
    const { stage, dismiss, openInfo } = stageSpy();
    const { pane } = mountPane({ stage });
    drag(pane, 0, -120);
    expect(openInfo).toHaveBeenCalledOnce();
    expect(dismiss).not.toHaveBeenCalled();
    drag(pane, 0, 300);
    expect(dismiss).toHaveBeenCalledOnce();
  });

  it('one tap does nothing and a double tap zooms in at the point', () => {
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

  it('a double tap on a zoomed photo fits it', () => {
    const onView = vi.fn();
    const { pane } = mountPane({ onView, view: ZOOMED });
    send(pane, 'pointerdown', 1, 700, 400);
    send(pane, 'pointerup', 1, 700, 400);
    send(pane, 'pointerdown', 2, 700, 400);
    send(pane, 'pointerup', 2, 700, 400);
    expect(onView).toHaveBeenLastCalledWith(FIT_VIEW);
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

  it('a two-finger tap toggles the chrome', () => {
    const { pane } = mountPane();
    send(pane, 'pointerdown', 1, 450, 400);
    send(pane, 'pointerdown', 2, 550, 400);
    send(pane, 'pointerup', 1, 450, 400);
    send(pane, 'pointerup', 2, 550, 400);
    expect(ui.fullscreen).toBe(true);
  });
});
