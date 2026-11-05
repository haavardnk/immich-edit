import type { Point } from '#lib/gestures/point.ts';

export const GRID_SIZE_HUD_MS = 900;

class GridSizeHud {
  visible = $state(false);
  at = $state<Point | null>(null);
  #timer: ReturnType<typeof setTimeout> | null = null;

  show = (at: Point | null = null): void => {
    this.at = at;
    this.visible = true;
    if (this.#timer !== null) clearTimeout(this.#timer);
    this.#timer = setTimeout(this.hide, GRID_SIZE_HUD_MS);
  };

  hide = (): void => {
    if (this.#timer !== null) clearTimeout(this.#timer);
    this.#timer = null;
    this.visible = false;
  };
}

export const gridSizeHud = new GridSizeHud();
