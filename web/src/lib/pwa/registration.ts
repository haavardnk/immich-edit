type Listener = () => void;

export type InstallingWorker = {
  readonly state: string;
  addEventListener(type: 'statechange', listener: Listener): void;
};

export type WatchedRegistration = {
  readonly installing: InstallingWorker | null;
  readonly waiting: unknown;
  addEventListener(type: 'updatefound', listener: Listener): void;
};

export function watchRegistration(
  reg: WatchedRegistration,
  hasController: () => boolean,
  onWaiting: () => void
): void {
  const watchInstalling = (): void => {
    const worker = reg.installing;
    if (!worker) return;
    worker.addEventListener('statechange', () => {
      if (worker.state === 'installed' && hasController()) onWaiting();
    });
  };
  if (reg.waiting && hasController()) onWaiting();
  watchInstalling();
  reg.addEventListener('updatefound', watchInstalling);
}

export function shouldReport(err: unknown): boolean {
  return !(err instanceof DOMException && err.name === 'SecurityError');
}

export class UpdateThrottle {
  #intervalMs: number;
  #last = Number.NEGATIVE_INFINITY;

  constructor(intervalMs: number) {
    this.#intervalMs = intervalMs;
  }

  due(now: number): boolean {
    return now - this.#last >= this.#intervalMs;
  }

  mark(now: number): void {
    this.#last = now;
  }
}
