import ComponentHost from '#lib/testing/ComponentHost.svelte';
import ContainerHarness from '#lib/testing/ContainerHarness.svelte';
import { flushSync, mount, tick, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({ mode: 'desktop' as 'desktop' | 'touch' }));

vi.mock('#lib/device/layout.svelte.ts', () => ({
  layout: {
    get mode() {
      return h.mode;
    },
    get hasKeyboard() {
      return false;
    }
  }
}));

type HarnessProps = ComponentProps<typeof ContainerHarness>;

const Host: Component<{ component: Component<HarnessProps>; props: HarnessProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

async function settle(): Promise<void> {
  flushSync();
  await tick();
  await new Promise((resolve) => setTimeout(resolve, 0));
  flushSync();
}

async function render(kind: HarnessProps['kind']) {
  const spies = { onClose: vi.fn(), onOpenChange: vi.fn(), onRow: vi.fn() };
  component = mount(Host, {
    target,
    props: { component: ContainerHarness, props: { kind, ...spies } }
  });
  await settle();
  return spies;
}

function button(name: string): HTMLButtonElement {
  const element = [...document.querySelectorAll<HTMLButtonElement>('button')].find(
    (candidate) =>
      candidate.getAttribute('aria-label') === name || candidate.textContent.trim() === name
  );
  if (!element) throw new Error(`no button ${name}`);
  return element;
}

function hasButton(name: string): boolean {
  return [...document.querySelectorAll('button')].some(
    (candidate) =>
      candidate.getAttribute('aria-label') === name || candidate.textContent.trim() === name
  );
}

function sheetHandle(): Element | null {
  return document.querySelector('button[data-sheet-handle]');
}

function popoverContent(): Element | null {
  return document.querySelector('[data-popover-content]');
}

function dialogName(dialog: Element): string {
  const id = dialog.getAttribute('aria-labelledby');
  return id ? (document.getElementById(id)?.textContent.trim() ?? '') : '';
}

function touchPointer(element: Element, type: string): void {
  element.dispatchEvent(
    new PointerEvent(type, {
      bubbles: true,
      cancelable: true,
      pointerId: 1,
      pointerType: 'touch',
      button: 0,
      buttons: type === 'pointerup' ? 0 : 1,
      clientX: 10,
      clientY: 10
    })
  );
}

beforeEach(() => {
  h.mode = 'desktop';
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  document.body.innerHTML = '';
  vi.useRealTimers();
});

describe('Dialog presentation', () => {
  it('keeps the centred modal on desktop', async () => {
    await render('dialog');
    expect(document.querySelector('[role="dialog"]')).not.toBeNull();
    expect(button('close')).toBeTruthy();
    expect(sheetHandle()).toBeNull();
  });

  it('becomes a bottom sheet named by its title on touch', async () => {
    h.mode = 'touch';
    const { onClose } = await render('dialog');
    const dialog = document.querySelector('[role="dialog"]');
    if (!dialog) throw new Error('no dialog');
    expect(dialogName(dialog)).toBe('Dialog title');
    expect(dialog.getAttribute('aria-modal')).toBe('true');
    expect(sheetHandle()).not.toBeNull();
    expect(dialog.textContent).toContain('Dialog body');
    button('close').click();
    await settle();
    expect(onClose).toHaveBeenCalled();
  });

  it('closes the touch sheet on Escape', async () => {
    h.mode = 'touch';
    const { onClose } = await render('dialog');
    (document.activeElement ?? document.body).dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })
    );
    await settle();
    expect(onClose).toHaveBeenCalled();
  });
});

describe('Popover presentation', () => {
  it('keeps the anchored popover on desktop', async () => {
    await render('menu');
    button('Toggle').click();
    await settle();
    expect(popoverContent()?.className).toContain('max-h-80');
    expect(sheetHandle()).toBeNull();
  });

  it('renders a panel exactly like a menu on desktop', async () => {
    await render('panel');
    button('Toggle').click();
    await settle();
    expect(popoverContent()?.className).toContain('max-h-80');
    expect(popoverContent()?.className).toContain('rounded-md');
    expect(hasButton('Done')).toBe(false);
  });

  it('opens a menu as an action sheet with Cancel on touch', async () => {
    h.mode = 'touch';
    const { onOpenChange, onRow } = await render('menu');
    expect(popoverContent()).toBeNull();
    button('Toggle').click();
    await settle();
    const content = popoverContent();
    expect(content?.getAttribute('role')).toBe('dialog');
    expect(content && dialogName(content)).toBe('Popover title');
    expect(content?.className).not.toContain('max-h-80');
    expect(sheetHandle()).toBeNull();
    expect(button('Cancel')).toBeTruthy();
    button('Run row').click();
    await settle();
    expect(onRow).toHaveBeenCalled();
    expect(onOpenChange).toHaveBeenLastCalledWith(false);
    expect(popoverContent()).toBeNull();
  });

  it('closes the action sheet from Cancel', async () => {
    h.mode = 'touch';
    const { onOpenChange, onRow } = await render('menu');
    button('Toggle').click();
    await settle();
    button('Cancel').click();
    await settle();
    expect(onRow).not.toHaveBeenCalled();
    expect(onOpenChange).toHaveBeenLastCalledWith(false);
  });

  it('opens a panel as a modal bottom sheet with Done on touch', async () => {
    h.mode = 'touch';
    const { onOpenChange } = await render('panel');
    button('Toggle').click();
    await settle();
    const content = popoverContent();
    expect(content?.getAttribute('aria-modal')).toBe('true');
    expect(content && dialogName(content)).toBe('Popover title');
    expect(sheetHandle()).not.toBeNull();
    expect(hasButton('Cancel')).toBe(false);
    expect(content?.querySelector('.max-h-80')).toBeNull();
    button('Done').click();
    await settle();
    expect(onOpenChange).toHaveBeenLastCalledWith(false);
    expect(popoverContent()).toBeNull();
  });

  it('opens a control popover as a non-modal sheet on touch', async () => {
    h.mode = 'touch';
    const { onOpenChange } = await render('control');
    button('Toggle').click();
    await settle();
    expect(popoverContent()?.getAttribute('role')).toBe('dialog');
    expect(popoverContent()?.getAttribute('aria-modal')).toBe('false');
    expect(document.querySelector('[data-dialog-overlay]')).toBeNull();
    expect(hasButton('Cancel')).toBe(false);
    button('Toggle').click();
    await settle();
    expect(onOpenChange).toHaveBeenLastCalledWith(false);
    expect(popoverContent()).toBeNull();
  });
});

describe('ContextMenu presentation', () => {
  it('opens the desktop menu on right click', async () => {
    await render('context');
    button('Thumb').dispatchEvent(
      new MouseEvent('contextmenu', { bubbles: true, cancelable: true, clientX: 10, clientY: 10 })
    );
    await settle();
    expect(document.querySelector('[role="menu"]')).not.toBeNull();
    expect(document.querySelector('[role="dialog"]')).toBeNull();
  });

  it('opens an action sheet on long press on touch and runs the row', async () => {
    h.mode = 'touch';
    const { onRow } = await render('context');
    vi.useFakeTimers();
    touchPointer(button('Thumb'), 'pointerdown');
    vi.advanceTimersByTime(600);
    flushSync();
    touchPointer(button('Thumb'), 'pointerup');
    vi.useRealTimers();
    await settle();
    expect(document.querySelector('[role="dialog"]')).not.toBeNull();
    expect(button('Cancel')).toBeTruthy();
    const row = document.querySelector<HTMLButtonElement>('button[data-menu-row]');
    expect(row?.textContent.trim()).toBe('Open row');
    row?.click();
    await settle();
    expect(onRow).toHaveBeenCalled();
    expect(document.querySelector('[role="dialog"]')).toBeNull();
  });

  it('opens the action sheet from the context menu key on touch', async () => {
    h.mode = 'touch';
    await render('context');
    const event = new MouseEvent('contextmenu', { bubbles: true, cancelable: true });
    button('Thumb').dispatchEvent(event);
    await settle();
    expect(event.defaultPrevented).toBe(true);
    expect(document.querySelector('[role="dialog"]')).not.toBeNull();
    button('Cancel').click();
    await settle();
    expect(document.querySelector('[role="dialog"]')).toBeNull();
  });

  it('does not open on a short touch press', async () => {
    h.mode = 'touch';
    await render('context');
    vi.useFakeTimers();
    touchPointer(button('Thumb'), 'pointerdown');
    vi.advanceTimersByTime(100);
    touchPointer(button('Thumb'), 'pointerup');
    vi.useRealTimers();
    await settle();
    expect(document.querySelector('[role="dialog"]')).toBeNull();
  });
});
