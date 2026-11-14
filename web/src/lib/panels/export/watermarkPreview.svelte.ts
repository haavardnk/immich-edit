import { untrack } from 'svelte';

class WatermarkPreview {
  private holders = $state(0);

  get showing(): boolean {
    return this.holders > 0;
  }

  hold(): () => void {
    untrack(() => (this.holders += 1));
    return () => {
      untrack(() => (this.holders -= 1));
    };
  }
}

export const watermarkPreview = new WatermarkPreview();
