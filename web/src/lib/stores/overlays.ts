import { isModalOpen } from '@immich/ui';

let held = 0;

export function holdModalSheet(): () => void {
  held += 1;
  let released = false;
  return () => {
    if (released) return;
    released = true;
    held -= 1;
  };
}

export function isAnyModalOpen(): boolean {
  return held > 0 || isModalOpen();
}
