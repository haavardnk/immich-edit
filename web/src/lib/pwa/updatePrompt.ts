import { SKIP_WAITING } from '#lib/pwa/protocol.ts';

const IDLE_POLL_MS = 250;

export type WaitingWorker = {
  postMessage(message: unknown): void;
};

export type UpdateDeps = {
  announce: (accept: () => void, dismissed: () => void) => void;
  waiting: () => WaitingWorker | null;
  busy: () => boolean;
  reload: () => void;
  sleep: (ms: number) => Promise<void>;
};

export async function whenIdle(busy: () => boolean, sleep: UpdateDeps['sleep']): Promise<void> {
  while (busy()) await sleep(IDLE_POLL_MS);
}

export class UpdatePrompt {
  #deps: UpdateDeps;
  #announced = false;
  #requested = false;
  #reloading = false;

  constructor(deps: UpdateDeps) {
    this.#deps = deps;
  }

  available(): void {
    if (this.#announced || this.#requested) return;
    this.#announced = true;
    this.#deps.announce(
      () => void this.accept(),
      () => {
        this.#announced = false;
      }
    );
  }

  async accept(): Promise<void> {
    if (this.#requested) return;
    this.#requested = true;
    await whenIdle(this.#deps.busy, this.#deps.sleep);
    const worker = this.#deps.waiting();
    if (worker) worker.postMessage(SKIP_WAITING);
    else this.#reload();
  }

  async controllerChanged(): Promise<void> {
    if (!this.#requested) return;
    await whenIdle(this.#deps.busy, this.#deps.sleep);
    this.#reload();
  }

  #reload(): void {
    if (this.#reloading) return;
    this.#reloading = true;
    this.#deps.reload();
  }
}
