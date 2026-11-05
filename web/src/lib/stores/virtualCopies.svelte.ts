import { createVirtualCopies } from '#lib/editor/copies.ts';

class VirtualCopiesStore {
  busy = $state(false);

  create = async (ids: string[]): Promise<void> => {
    if (this.busy || ids.length === 0) return;
    this.busy = true;
    try {
      await createVirtualCopies(ids);
    } finally {
      this.busy = false;
    }
  };
}

export const virtualCopies = new VirtualCopiesStore();
