import { listWatermarks, type WatermarkMeta } from '#lib/api/watermarks.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

class WatermarkLibrary {
  items = $state<WatermarkMeta[]>([]);

  async load(): Promise<void> {
    try {
      this.items = await listWatermarks();
    } catch (e) {
      this.items = [];
      toasts.fail('watermarks', e);
    }
  }

  has(id: string): boolean {
    return this.items.some((item) => item.id === id);
  }

  nameOf(id: string | null): string | null {
    return this.items.find((item) => item.id === id)?.name ?? null;
  }
}

export const watermarkLibrary = new WatermarkLibrary();
