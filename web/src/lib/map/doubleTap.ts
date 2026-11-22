export type Tap = 'single' | 'double' | 'ignore';

export class DoubleTap {
  #last = Number.NEGATIVE_INFINITY;
  #quietUntil = Number.NEGATIVE_INFINITY;
  readonly #window: number;

  constructor(window: number) {
    this.#window = window;
  }

  tap(now: number, detail = 1): Tap {
    if (now < this.#quietUntil) return 'ignore';
    if (detail >= 2 || now - this.#last <= this.#window) {
      this.#last = Number.NEGATIVE_INFINITY;
      this.#quietUntil = now + this.#window;
      return 'double';
    }
    this.#last = now;
    return 'single';
  }
}
