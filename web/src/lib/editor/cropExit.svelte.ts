import { cubicOut } from 'svelte/easing';
import { Tween } from 'svelte/motion';

class CropExit {
  #scale = new Tween(1, { duration: 250, easing: cubicOut });

  get transform(): string {
    const scale = this.#scale.current;
    return scale === 1 ? '' : ` transform: scale(${scale});`;
  }

  hand(scale: number | null): void {
    if (scale === null || !Number.isFinite(scale) || scale <= 0) return;
    void this.#scale.set(scale, { duration: 0 });
    void this.#scale.set(1);
  }
}

export const cropExit = new CropExit();
