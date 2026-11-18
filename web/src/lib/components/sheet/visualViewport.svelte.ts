import {
  NO_INSET,
  keyboardInset,
  type ViewportInset
} from '#lib/components/sheet/viewportInset.ts';

type ViewportEvent = 'resize' | 'scroll';

export type ViewportSource = {
  height: number;
  offsetTop: number;
  addEventListener: (type: ViewportEvent, listener: () => void) => void;
  removeEventListener: (type: ViewportEvent, listener: () => void) => void;
};

export type ViewportHost = { innerHeight: number; visualViewport: ViewportSource | null };

export type ViewportInsetTracker = ViewportInset & { watch: (host: ViewportHost) => () => void };

const EVENTS: readonly ViewportEvent[] = ['resize', 'scroll'];

function sourceOf(viewport: VisualViewport): ViewportSource {
  return {
    get height() {
      return viewport.height;
    },
    get offsetTop() {
      return viewport.offsetTop;
    },
    addEventListener: (type, listener) => viewport.addEventListener(type, listener),
    removeEventListener: (type, listener) => viewport.removeEventListener(type, listener)
  };
}

export function windowViewport(): ViewportHost {
  const viewport = window.visualViewport;
  return {
    get innerHeight() {
      return window.innerHeight;
    },
    visualViewport: viewport ? sourceOf(viewport) : null
  };
}

export function viewportInset(): ViewportInsetTracker {
  let inset = $state<ViewportInset>(NO_INSET);

  function watch(host: ViewportHost): () => void {
    const viewport = host.visualViewport;
    if (!viewport) return () => {};

    const update = (): void => {
      inset = keyboardInset({
        innerHeight: host.innerHeight,
        height: viewport.height,
        offsetTop: viewport.offsetTop
      });
    };

    for (const event of EVENTS) viewport.addEventListener(event, update);
    update();
    return () => {
      for (const event of EVENTS) viewport.removeEventListener(event, update);
      inset = NO_INSET;
    };
  }

  return {
    get top() {
      return inset.top;
    },
    get bottom() {
      return inset.bottom;
    },
    watch
  };
}
