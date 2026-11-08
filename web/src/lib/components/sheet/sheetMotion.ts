import { afterTransition } from '#lib/components/sheet/transitions.ts';
import { dragFrame, snapSheet } from '#lib/components/sheet/sheetSnap.ts';
import type { PanEnd, PanUpdate } from '#lib/gestures/pan.ts';
import { untrack } from 'svelte';
import type { Attachment } from 'svelte/attachments';

export type SheetMotionOptions = {
  heights: () => readonly number[];
  rest: () => number;
  available: () => number;
  dismissable: () => boolean;
  grab: () => void;
  snap: (index: number) => void;
  dismiss: () => void;
};

export type SheetMotion = {
  panel: Attachment<HTMLElement>;
  dragging: () => boolean;
  rest: (height: number) => void;
  settle: (height: number) => Promise<void>;
  slideOut: () => Promise<void>;
  begin: (pan: PanUpdate) => void;
  follow: (pan: PanUpdate) => void;
  release: (pan: PanEnd) => void;
  abort: () => void;
};

export function sheetMotion(options: SheetMotionOptions): SheetMotion {
  let panel: HTMLElement | null = null;
  let dragging = false;
  let startHeight = 0;
  let generation = 0;

  function place(height: number, offset: number): void {
    if (!panel) return;
    panel.style.height = height > 0 ? `${Math.min(height, options.available())}px` : '';
    panel.style.transform = offset > 0 ? `translate3d(0, ${offset}px, 0)` : '';
  }

  function animate(height: number, offset: number): Promise<void> {
    const element = panel;
    if (!element) return Promise.resolve();
    const mine = ++generation;
    element.dataset.snapping = '';
    place(height, offset);
    return afterTransition(element).then(() => {
      if (mine === generation) delete element.dataset.snapping;
    });
  }

  function finish(): void {
    dragging = false;
    if (panel) panel.style.transition = '';
  }

  function begin(pan: PanUpdate): void {
    if (!panel || pan.axis !== 'y') return;
    dragging = true;
    generation += 1;
    delete panel.dataset.snapping;
    options.grab();
    startHeight = panel.getBoundingClientRect().height;
    panel.style.transition = 'none';
    follow(pan);
  }

  function follow(pan: PanUpdate): void {
    if (!dragging) return;
    const frame = dragFrame(startHeight - pan.dy, options.heights(), options.dismissable());
    place(frame.height, frame.offset);
  }

  function release(pan: PanEnd): void {
    if (!dragging) return;
    finish();
    const heights = options.heights();
    const target = snapSheet({
      heights,
      height: startHeight - pan.dy,
      velocity: pan.vy,
      dismissable: options.dismissable()
    });
    if (target === 'dismiss') {
      options.dismiss();
      return;
    }
    options.snap(target);
    void animate(heights[target] ?? 0, 0);
  }

  function abort(): void {
    if (!dragging) return;
    finish();
    void animate(options.rest(), 0);
  }

  return {
    panel: (element) => {
      panel = element;
      untrack(() => place(options.rest(), 0));
      return () => {
        if (panel === element) panel = null;
      };
    },
    dragging: () => dragging,
    rest: (height) => place(height, 0),
    settle: (height) => {
      if (panel) delete panel.dataset.leaving;
      return animate(height, 0);
    },
    slideOut: () => {
      if (!panel) return Promise.resolve();
      panel.dataset.leaving = '';
      return animate(options.rest(), Math.max(1, panel.getBoundingClientRect().height));
    },
    begin,
    follow,
    release,
    abort
  };
}
