import { isModalOpen } from '@immich/ui';

let held = 0;
let escapeHolds = 0;
const taken = new WeakSet<Event>();

function takeEscape(e: KeyboardEvent): void {
  if (e.key === 'Escape') taken.add(e);
}

export function holdModalSheet(): () => void {
  held += 1;
  let released = false;
  return () => {
    if (released) return;
    released = true;
    held -= 1;
  };
}

export function holdEscape(): () => void {
  if (escapeHolds++ === 0) document.addEventListener('keydown', takeEscape, { capture: true });
  let released = false;
  return () => {
    if (released) return;
    released = true;
    if (--escapeHolds === 0) {
      document.removeEventListener('keydown', takeEscape, { capture: true });
    }
  };
}

export function escapeTaken(e: KeyboardEvent): boolean {
  return taken.has(e);
}

export function isAnyModalOpen(): boolean {
  return held > 0 || isModalOpen();
}
