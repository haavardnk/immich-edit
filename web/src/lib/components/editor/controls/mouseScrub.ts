import { pointerDrag } from '#lib/gestures/drag.ts';
import type { Attachment } from 'svelte/attachments';

const DOUBLE_CLICK_GUARD_MS = 500;

export type MouseScrubSpec = {
  enabled: () => boolean;
  slop: number;
  start: (e: PointerEvent) => boolean;
  move: (stepX: number, clientY: number) => void;
  end: () => void;
};

type Scrub = { origin: number; last: number; active: boolean; moved: boolean };

export function mouseScrub(spec: MouseScrubSpec): Attachment<HTMLElement> {
  return (element) => {
    let endedAt = -Infinity;
    let swallow = false;

    function finish(e: PointerEvent, scrub: Scrub): void {
      if (!scrub.active) return;
      spec.end();
      if (!scrub.moved) return;
      endedAt = e.timeStamp;
      swallow = true;
      setTimeout(() => {
        swallow = false;
      });
    }

    const drag = pointerDrag<Scrub>({
      start(e) {
        if (e.pointerType !== 'mouse' || e.button !== 0 || !spec.enabled()) return null;
        const scrub = { origin: e.clientX, last: e.clientX, active: false, moved: false };
        if (spec.slop > 0) return scrub;
        if (!spec.start(e)) return null;
        scrub.active = true;
        return scrub;
      },
      move(e, scrub) {
        if (!scrub.active) {
          if (Math.abs(e.clientX - scrub.origin) <= spec.slop || !spec.start(e)) return;
          scrub.active = true;
        }
        scrub.moved = true;
        spec.move(e.clientX - scrub.last, e.clientY);
        scrub.last = e.clientX;
      },
      end: finish,
      cancel: finish
    });

    function click(e: MouseEvent): void {
      const repeat = e.type === 'dblclick' && e.timeStamp - endedAt < DOUBLE_CLICK_GUARD_MS;
      if (!swallow && !repeat) return;
      e.preventDefault();
      e.stopPropagation();
    }

    const listening = new AbortController();
    const options = { signal: listening.signal };
    element.addEventListener('pointerdown', (e) => void drag.begin(e), options);
    element.addEventListener('pointermove', drag.follow.onpointermove, options);
    element.addEventListener('pointerup', drag.follow.onpointerup, options);
    element.addEventListener('pointercancel', drag.follow.onpointercancel, options);
    element.addEventListener('lostpointercapture', drag.follow.onlostpointercapture, options);
    element.addEventListener('click', click, { ...options, capture: true });
    element.addEventListener('dblclick', click, { ...options, capture: true });

    return () => listening.abort();
  };
}
