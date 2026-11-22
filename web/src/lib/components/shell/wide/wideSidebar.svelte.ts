import { readStored, writeStored } from '#lib/utils/storage.ts';

const KEY = 'immich-edit:wideSidebar';

class WideSidebar {
  open = $state(readStored<{ open: boolean }>(KEY)?.open !== false);

  toggle = (): void => {
    this.open = !this.open;
    writeStored(KEY, { open: this.open });
  };
}

export const wideSidebar = new WideSidebar();
