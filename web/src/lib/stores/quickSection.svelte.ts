import { readStored, writeStored } from '#lib/utils/storage.ts';

const STORAGE_KEY = 'immich-edit:quickSection';

class QuickSectionStore {
  open = $state(readStored<{ open?: boolean }>(STORAGE_KEY)?.open !== false);

  setOpen = (open: boolean): void => {
    this.open = open;
    writeStored(STORAGE_KEY, { open });
  };
}

export const quickSection = new QuickSectionStore();
