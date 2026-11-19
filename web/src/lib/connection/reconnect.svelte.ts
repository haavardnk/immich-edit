const LONGEST_DELAY_MS = 10000;
const RETRY_DELAYS_MS: readonly number[] = [1000, 2000, 4000, 8000, LONGEST_DELAY_MS];

const TICK_MS = 1000;

export function retrySeconds(attempt: number): number {
  return (RETRY_DELAYS_MS[attempt] ?? LONGEST_DELAY_MS) / TICK_MS;
}

export class Reconnect {
  attempt = $state(0);
  secondsLeft = $state(retrySeconds(0));
  probing = $state(false);
  offline = $state(false);

  readonly #probe: () => Promise<boolean>;
  readonly #onBack: () => void;

  constructor(probe: () => Promise<boolean>, onBack: () => void) {
    this.#probe = probe;
    this.#onBack = onBack;
  }

  start(): () => void {
    this.offline = !navigator.onLine;
    const timer = setInterval(() => this.#tick(), TICK_MS);
    const goOffline = (): void => {
      this.offline = true;
    };
    const goOnline = (): void => {
      this.offline = false;
      void this.retryNow();
    };
    window.addEventListener('offline', goOffline);
    window.addEventListener('online', goOnline);
    return () => {
      clearInterval(timer);
      window.removeEventListener('offline', goOffline);
      window.removeEventListener('online', goOnline);
    };
  }

  retryNow = async (): Promise<void> => {
    if (this.probing) return;
    this.attempt = 0;
    await this.#run();
  };

  #tick(): void {
    if (this.offline || this.probing) return;
    this.secondsLeft -= 1;
    if (this.secondsLeft <= 0) void this.#run();
  }

  async #run(): Promise<void> {
    this.probing = true;
    const back = await this.#probe().catch(() => false);
    this.probing = false;
    if (back) {
      this.#onBack();
      return;
    }
    this.attempt += 1;
    this.secondsLeft = retrySeconds(this.attempt);
  }
}
