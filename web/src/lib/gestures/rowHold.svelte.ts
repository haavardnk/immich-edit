import { clickGuard } from '#lib/gestures/clickGuard.ts';
import { gestures } from '#lib/gestures/gestures.ts';
import { longPressRecognizer } from '#lib/gestures/longPress.ts';
import type { Point } from '#lib/gestures/point.ts';
import type { Attachment } from 'svelte/attachments';

const HOLD_MS = 350;

export interface RowHoldSpec {
  attribute: string;
  start: (key: string) => void;
  end: () => void;
}

export interface RowHold {
  readonly held: string | null;
  attachment: Attachment<HTMLElement>;
}

export function rowHold(spec: RowHoldSpec): RowHold {
  const guard = clickGuard();
  let held = $state<string | null>(null);
  let row: Element | null = null;

  function rowAt({ x, y }: Point): Element | null {
    return document.elementFromPoint(x, y)?.closest(`[${spec.attribute}]`) ?? null;
  }

  function inside({ x, y }: Point): boolean {
    const box = row?.getBoundingClientRect();
    return !!box && x >= box.left && x <= box.right && y >= box.top && y <= box.bottom;
  }

  function release(): void {
    if (held === null) return;
    held = null;
    row = null;
    spec.end();
  }

  const hold = gestures({
    recognizers: [
      longPressRecognizer({
        thresholds: { longPressMs: HOLD_MS },
        accepts: (point) => rowAt(point)?.getAttribute(spec.attribute) != null,
        start: (point) => {
          row = rowAt(point);
          const key = row?.getAttribute(spec.attribute);
          if (key == null) return;
          guard.arm();
          held = key;
          spec.start(key);
        },
        move: (point) => {
          if (!inside(point)) release();
        },
        end: release,
        cancel: release
      })
    ],
    touchAction: null
  });

  return {
    get held() {
      return held;
    },
    attachment: (element) => {
      const stopHold = hold(element);
      const stopGuard = guard.attachment(element);
      return () => {
        if (typeof stopHold === 'function') stopHold();
        if (typeof stopGuard === 'function') stopGuard();
        release();
      };
    }
  };
}
