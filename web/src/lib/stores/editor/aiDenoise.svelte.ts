import { getDenoiseStatus, startDenoise, type DenoiseStatus } from '$lib/api/denoise';
import { errorMessage } from '$lib/utils/errors';

const POLL_MS = 1500;

function waiting(status: DenoiseStatus | null): boolean {
  return status?.state === 'queued' || status?.state === 'running';
}

function pageHidden(): boolean {
  return typeof document !== 'undefined' && document.visibilityState === 'hidden';
}

export class AiDenoiseTracker {
  status = $state<DenoiseStatus | null>(null);
  private landed = $state(0);
  private assetId: string | null = null;
  private enabled = false;
  private inFlight = false;
  private paused = false;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private generation = 0;
  private readonly onReady: () => void;

  constructor(onReady: () => void) {
    this.onReady = onReady;
  }

  get revision(): string {
    return this.landed === 0 ? '' : `denoised-${this.landed}`;
  }

  sync(assetId: string | null, active: boolean): void {
    if (assetId !== this.assetId) this.reset(assetId);
    const turnedOn = active && !this.enabled;
    this.enabled = active;
    if (!assetId || !active) {
      this.stopPolling();
      return;
    }
    if (this.inFlight) return;
    const state = this.status?.state;
    if (state === undefined || state === 'idle' || (turnedOn && state === 'model_missing')) {
      void this.request(assetId, true);
    } else if (waiting(this.status) && this.timer === null && !this.paused) {
      this.schedule(assetId);
    }
  }

  retry(): void {
    if (this.assetId && !this.inFlight) void this.request(this.assetId, true);
  }

  reset(assetId: string | null = null): void {
    this.stopPolling();
    this.generation++;
    this.assetId = assetId;
    this.enabled = false;
    this.inFlight = false;
    this.status = null;
    this.landed = 0;
  }

  private async request(assetId: string, start: boolean): Promise<void> {
    const generation = this.generation;
    this.inFlight = true;
    let next: DenoiseStatus;
    try {
      next = start ? await startDenoise(assetId) : await getDenoiseStatus(assetId);
    } catch (err) {
      next = { state: 'failed', error: errorMessage(err) };
    }
    if (generation !== this.generation) return;
    this.inFlight = false;
    const wasPending = this.status !== null && this.status.state !== 'ready';
    this.status = next;
    if (next.state === 'ready') {
      if (!wasPending) return;
      this.landed++;
      this.onReady();
      return;
    }
    if (!this.enabled) return;
    if (waiting(next)) this.schedule(assetId);
    else if (next.state === 'idle' && !start) void this.request(assetId, true);
  }

  private schedule(assetId: string): void {
    this.stopPolling();
    this.timer = setTimeout(() => {
      this.timer = null;
      if (pageHidden()) {
        this.paused = true;
        document.addEventListener('visibilitychange', this.onVisibilityChange);
        return;
      }
      void this.request(assetId, false);
    }, POLL_MS);
  }

  private onVisibilityChange = (): void => {
    if (pageHidden()) return;
    const assetId = this.assetId;
    this.stopPolling();
    if (assetId && this.enabled) void this.request(assetId, false);
  };

  private stopPolling(): void {
    if (this.paused) {
      this.paused = false;
      document.removeEventListener('visibilitychange', this.onVisibilityChange);
    }
    if (this.timer === null) return;
    clearTimeout(this.timer);
    this.timer = null;
  }
}
