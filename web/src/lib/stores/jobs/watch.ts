import { ApiError, url } from '#lib/api/client.ts';
import { getJob, type Job } from '#lib/api/jobs.ts';

const POLL_BASE_MS = 4000;
const MAX_BACKOFF_STEPS = 4;
const MAX_STREAM_ERRORS = 5;

export function pollDelay(failures: number): number {
  return POLL_BASE_MS * 2 ** Math.min(failures, MAX_BACKOFF_STEPS);
}

function isGone(error: unknown): boolean {
  return error instanceof ApiError && (error.status === 404 || error.status === 410);
}

export interface JobWatchHost {
  receive(job: Job): void;
  isActive(id: string): boolean;
}

export class JobWatch {
  #host: JobWatchHost;
  #sources = new Map<string, EventSource>();
  #streamErrors = new Map<string, number>();
  #fallbacks = new Map<string, ReturnType<typeof setTimeout>>();

  constructor(host: JobWatchHost) {
    this.#host = host;
  }

  connect(id: string): void {
    if (this.#sources.has(id)) return;
    if ((this.#streamErrors.get(id) ?? 0) >= MAX_STREAM_ERRORS) return;
    const source = new EventSource(url`/api/jobs/${id}/events`);
    source.addEventListener('job', (ev) => {
      let job: Job;
      try {
        job = JSON.parse((ev as MessageEvent).data) as Job;
      } catch {
        return;
      }
      this.#streamErrors.delete(id);
      this.#host.receive(job);
    });
    source.onerror = () => {
      const errors = (this.#streamErrors.get(id) ?? 0) + 1;
      this.#streamErrors.set(id, errors);
      if (errors >= MAX_STREAM_ERRORS) {
        this.#disconnect(id);
        this.#poll(id, 0);
        return;
      }
      if (source.readyState === EventSource.CLOSED) {
        this.#sources.delete(id);
      }
    };
    this.#sources.set(id, source);
  }

  resume(id: string): void {
    this.forget(id);
    this.connect(id);
  }

  stop(id: string): void {
    this.#disconnect(id);
    this.#stopFallback(id);
  }

  forget(id: string): void {
    this.stop(id);
    this.#streamErrors.delete(id);
  }

  #disconnect(id: string): void {
    const source = this.#sources.get(id);
    if (!source) return;
    source.close();
    this.#sources.delete(id);
  }

  #poll(id: string, failures: number): void {
    this.#stopFallback(id);
    const timer = setTimeout(() => void this.#check(id, failures), pollDelay(failures));
    this.#fallbacks.set(id, timer);
  }

  async #check(id: string, failures: number): Promise<void> {
    this.#fallbacks.delete(id);
    let job: Job;
    try {
      job = (await getJob(id)).job;
    } catch (e) {
      if (!isGone(e)) this.#poll(id, failures + 1);
      return;
    }
    this.#host.receive(job);
    if (this.#host.isActive(id)) this.#poll(id, 0);
  }

  #stopFallback(id: string): void {
    const timer = this.#fallbacks.get(id);
    if (timer === undefined) return;
    clearTimeout(timer);
    this.#fallbacks.delete(id);
  }
}
