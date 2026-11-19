import type { CullKind } from '#lib/browse/cullMark.ts';
import type { CullState } from '#lib/browse/cullState.ts';

export const ADVANCE_MS = 700;
const LINGER_MS = 1200;

export interface HudMark {
  id: string;
  kind: CullKind;
  state: CullState;
  advancing: boolean;
  serial: number;
}

class RatingHud {
  shown = $state.raw<HudMark | null>(null);
  #timer: ReturnType<typeof setTimeout> | null = null;
  #serial = 0;

  show(id: string, kind: CullKind, state: CullState, step: (() => void) | null): void {
    this.#stop();
    this.#serial += 1;
    this.shown = { id, kind, state, advancing: step !== null, serial: this.#serial };
    this.#timer = setTimeout(
      () => {
        this.#timer = null;
        this.shown = null;
        step?.();
      },
      step ? ADVANCE_MS : LINGER_MS
    );
  }

  follow(id: string | null): void {
    if (this.shown && this.shown.id !== id) this.clear();
  }

  clear(): void {
    this.#stop();
    this.shown = null;
  }

  #stop(): void {
    if (this.#timer) clearTimeout(this.#timer);
    this.#timer = null;
  }
}

export const ratingHud = new RatingHud();
