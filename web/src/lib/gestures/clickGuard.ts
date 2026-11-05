import type { Attachment } from 'svelte/attachments';

export type ClickGuard = { arm: () => void; attachment: Attachment<HTMLElement> };

export function clickGuard(): ClickGuard {
  let armed = false;

  function disarm(): void {
    armed = false;
  }

  function swallow(event: MouseEvent): void {
    if (!armed || event.detail === 0) return;
    disarm();
    event.preventDefault();
    event.stopImmediatePropagation();
  }

  function release(event: TouchEvent): void {
    if (!armed || !event.cancelable) return;
    disarm();
    event.preventDefault();
  }

  return {
    arm: () => {
      armed = true;
    },
    attachment: (element) => {
      const listening = new AbortController();
      const options = { capture: true, signal: listening.signal };
      element.addEventListener('pointerdown', disarm, options);
      element.addEventListener('click', swallow, options);
      element.addEventListener('touchend', release, options);
      return () => {
        listening.abort();
        disarm();
      };
    }
  };
}
