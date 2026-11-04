import ComponentHost from '#lib/testing/ComponentHost.svelte';
import SheetHarness from '#lib/testing/SheetHarness.svelte';
import { isAnyModalOpen } from '#lib/stores/overlays.ts';
import { flushSync, mount, tick, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

type HarnessProps = ComponentProps<typeof SheetHarness>;

const Host: Component<{ component: Component<HarnessProps>; props: HarnessProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;
let frames: Map<number, FrameRequestCallback>;
let lastFrame: number;

async function settle(): Promise<void> {
  flushSync();
  await tick();
  await new Promise((resolve) => setTimeout(resolve, 0));
  flushSync();
}

async function render(props: Partial<HarnessProps> = {}) {
  const onOpenChange = vi.fn();
  component = mount(Host, {
    target,
    props: { component: SheetHarness, props: { onOpenChange, ...props } }
  });
  await settle();
  return { onOpenChange };
}

function sheet(): HTMLElement {
  const element = document.querySelector<HTMLElement>('[role="dialog"]');
  if (!element) throw new Error('no sheet');
  return element;
}

function handle(): HTMLButtonElement {
  const element = document.querySelector<HTMLButtonElement>('button[data-sheet-handle]');
  if (!element) throw new Error('no handle');
  return element;
}

function button(name: string): HTMLButtonElement {
  const element = [...document.querySelectorAll<HTMLButtonElement>('button')].find(
    (candidate) =>
      candidate.getAttribute('aria-label') === name || candidate.textContent.trim() === name
  );
  if (!element) throw new Error(`no button ${name}`);
  return element;
}

function describedValue(element: HTMLElement): string {
  const id = element.getAttribute('aria-describedby');
  return id ? (document.getElementById(id)?.textContent.trim() ?? '') : '';
}

function pressEscape(): void {
  document.activeElement?.dispatchEvent(
    new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })
  );
  flushSync();
}

function runFrames(): void {
  const callbacks = [...frames.values()];
  frames.clear();
  for (const callback of callbacks) callback(0);
}

function touch(element: Element, type: string, clientY: number): void {
  element.dispatchEvent(
    new PointerEvent(type, {
      bubbles: true,
      cancelable: true,
      pointerId: 7,
      pointerType: 'touch',
      button: 0,
      buttons: type === 'pointerup' ? 0 : 1,
      clientX: 40,
      clientY
    })
  );
}

function dragHandle(from: number, to: number): void {
  const grip = handle();
  touch(grip, 'pointerdown', from);
  touch(grip, 'pointermove', from + (to - from) / 2);
  runFrames();
  touch(grip, 'pointermove', to);
  runFrames();
  touch(grip, 'pointerup', to);
  flushSync();
}

beforeEach(() => {
  frames = new Map();
  lastFrame = 0;
  vi.stubGlobal(
    'requestAnimationFrame',
    vi.fn((callback: FrameRequestCallback) => {
      lastFrame += 1;
      frames.set(lastFrame, callback);
      return lastFrame;
    })
  );
  vi.stubGlobal(
    'cancelAnimationFrame',
    vi.fn((id: number) => frames.delete(id))
  );
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  document.body.innerHTML = '';
  vi.unstubAllGlobals();
});

describe('BottomSheet', () => {
  it('is a dialog named by its title with a resize handle', async () => {
    await render();
    const labelledBy = sheet().getAttribute('aria-labelledby');
    expect(labelledBy && document.getElementById(labelledBy)?.textContent.trim()).toBe(
      'Sheet title'
    );
    expect(handle().getAttribute('aria-label')).toBe('Resize sheet');
    expect(handle().type).toBe('button');
  });

  it('hides the handle from assistive tech with a single detent', async () => {
    await render();
    expect(handle().getAttribute('aria-hidden')).toBe('true');
    expect(handle().tabIndex).toBe(-1);
  });

  it('steps to the next detent when the handle is tapped', async () => {
    await render({ detents: [{ fraction: 0.5 }, { fraction: 1 }], detentLabels: ['Half', 'Full'] });
    const output = document.querySelector('[data-testid="detent"]');
    expect(describedValue(handle())).toBe('Half');
    handle().click();
    flushSync();
    expect(describedValue(handle())).toBe('Full');
    expect(output?.textContent).toBe('1');
    handle().click();
    flushSync();
    expect(describedValue(handle())).toBe('Half');
    expect(output?.textContent).toBe('0');
  });

  it('closes from its close button', async () => {
    const { onOpenChange } = await render();
    button('close').click();
    await settle();
    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(document.querySelector('[role="dialog"]')).toBeNull();
  });

  it('closes a modal sheet on Escape', async () => {
    const { onOpenChange } = await render();
    expect(sheet().getAttribute('aria-modal')).toBe('true');
    pressEscape();
    await settle();
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it('stays open when the parent refuses to close on Escape', async () => {
    const { onOpenChange } = await render({ refuse: true });
    pressEscape();
    await settle();
    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(sheet()).toBeTruthy();
    expect(handle()).toBeTruthy();
  });

  it('ignores Escape when it is not dismissable', async () => {
    const { onOpenChange } = await render({ dismissable: false });
    pressEscape();
    await settle();
    expect(onOpenChange).not.toHaveBeenCalled();
    expect(sheet()).toBeTruthy();
  });

  it('dismisses when the handle is dragged down past the threshold', async () => {
    const { onOpenChange } = await render();
    dragHandle(100, 400);
    await settle();
    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(document.querySelector('[role="dialog"]')).toBeNull();
  });

  it('does not dismiss a sheet that is not dismissable when dragged', async () => {
    const { onOpenChange } = await render({ dismissable: false });
    dragHandle(100, 400);
    await settle();
    expect(onOpenChange).not.toHaveBeenCalled();
    expect(sheet()).toBeTruthy();
  });

  it('swallows the click that ends a drag', async () => {
    await render({ detents: [{ fraction: 0.5 }, { fraction: 1 }], dismissable: false });
    const output = document.querySelector('[data-testid="detent"]');
    dragHandle(100, 60);
    await settle();
    const settled = output?.textContent;
    handle().dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, detail: 1 }));
    flushSync();
    expect(output?.textContent).toBe(settled);
  });

  it('dims the page behind a modal sheet', async () => {
    await render();
    expect(document.querySelector('[data-dialog-overlay]')).not.toBeNull();
  });

  it('holds the modal flag until it unmounts while open', async () => {
    await render();
    expect(isAnyModalOpen()).toBe(true);
    if (component) await unmount(component);
    component = null;
    await settle();
    expect(isAnyModalOpen()).toBe(false);
  });

  it('leaves the page usable behind a non-modal sheet', async () => {
    await render({ modal: false });
    expect(sheet().getAttribute('aria-modal')).toBe('false');
    expect(document.querySelector('[data-dialog-overlay]')).toBeNull();
    expect(isAnyModalOpen()).toBe(false);
    const outside = button('Outside');
    outside.focus();
    await settle();
    expect(document.activeElement).toBe(outside);
    button('Inside').focus();
    await settle();
    outside.focus();
    await settle();
    expect(document.activeElement).toBe(outside);
  });
});
