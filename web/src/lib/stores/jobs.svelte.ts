import { cancelJob, clearJobs, getJob, listJobs, type Job, type JobItem } from '$lib/api/jobs';
import { url } from '$lib/api/client';
import { editedThumbs } from '$lib/stores/editedThumbs.svelte';
import { toasts } from '$lib/stores/toasts.svelte';

function isActive(status: Job['status']): boolean {
  return status === 'pending' || status === 'running';
}

const POLL_BASE_MS = 4000;
const MAX_BACKOFF_STEPS = 4;
const MAX_STREAM_ERRORS = 5;

class JobsStore {
  jobs = $state<Job[]>([]);
  open = $state(false);
  loading = $state(false);
  items = $state<Record<string, JobItem[]>>({});

  private sources = new Map<string, EventSource>();
  private streamErrors = new Map<string, number>();
  private pollTimer: ReturnType<typeof setTimeout> | null = null;
  private polling = false;
  private failures = 0;

  activeCount = $derived(this.jobs.filter((j) => isActive(j.status)).length);
  clearableCount = $derived(this.jobs.filter((j) => !isActive(j.status)).length);

  load = async (): Promise<void> => {
    if (this.loading) return;
    this.loading = true;
    try {
      this.jobs = await listJobs();
      this.failures = 0;
      this.syncStreams();
    } catch (e) {
      this.failures += 1;
      if (this.failures === 1) toasts.fail('Failed to load jobs', e, 6000);
    } finally {
      this.loading = false;
    }
  };

  toggle = (): void => {
    if (this.open) {
      this.close();
    } else {
      this.open = true;
      this.failures = 0;
      this.startPolling();
    }
  };

  close = (): void => {
    this.open = false;
    this.stopPolling();
  };

  cancel = async (id: string): Promise<void> => {
    try {
      await cancelJob(id);
    } catch (e) {
      toasts.fail('Failed to cancel job', e, 6000);
    }
  };

  clear = async (): Promise<void> => {
    try {
      await clearJobs();
      const removed = new Set(this.jobs.filter((j) => !isActive(j.status)).map((j) => j.id));
      for (const id of removed) {
        this.disconnect(id);
        this.streamErrors.delete(id);
      }
      this.jobs = this.jobs.filter((j) => isActive(j.status));
      const items = { ...this.items };
      for (const id of removed) delete items[id];
      this.items = items;
    } catch (e) {
      toasts.fail('Failed to clear jobs', e, 6000);
    }
  };

  loadItems = async (id: string): Promise<void> => {
    try {
      const detail = await getJob(id);
      this.patch(detail.job);
      this.items = { ...this.items, [id]: detail.items };
    } catch (e) {
      toasts.fail('Failed to load job items', e, 6000);
    }
  };

  private patch(job: Job): void {
    const idx = this.jobs.findIndex((j) => j.id === job.id);
    if (idx === -1) {
      this.jobs = [job, ...this.jobs];
    } else {
      const next = [...this.jobs];
      next[idx] = job;
      this.jobs = next;
    }
  }

  private syncStreams(): void {
    for (const job of this.jobs) {
      if (isActive(job.status)) {
        this.connect(job.id);
      } else {
        this.disconnect(job.id);
      }
    }
  }

  private connect(id: string): void {
    if (this.sources.has(id)) return;
    if ((this.streamErrors.get(id) ?? 0) >= MAX_STREAM_ERRORS) return;
    const source = new EventSource(url`/api/jobs/${id}/events`);
    source.addEventListener('job', (ev) => {
      try {
        const job = JSON.parse((ev as MessageEvent).data) as Job;
        this.streamErrors.delete(id);
        this.patch(job);
        if (!isActive(job.status)) {
          if (
            (job.kind === 'apply_preset' ||
              job.kind === 'paste_edits' ||
              job.kind === 'reset_edits') &&
            job.status === 'completed'
          ) {
            void editedThumbs.refresh();
          }
          this.disconnect(job.id);
        }
      } catch {
        /* ignore malformed event */
      }
    });
    source.onerror = () => {
      const errors = (this.streamErrors.get(id) ?? 0) + 1;
      this.streamErrors.set(id, errors);
      if (errors >= MAX_STREAM_ERRORS) {
        this.disconnect(id);
        return;
      }
      if (source.readyState === EventSource.CLOSED) {
        this.sources.delete(id);
      }
    };
    this.sources.set(id, source);
  }

  private disconnect(id: string): void {
    const source = this.sources.get(id);
    if (source) {
      source.close();
      this.sources.delete(id);
    }
  }

  private startPolling(): void {
    if (this.polling) return;
    this.polling = true;
    void this.poll();
  }

  private async poll(): Promise<void> {
    await this.load();
    if (!this.polling) return;
    const delay = POLL_BASE_MS * 2 ** Math.min(this.failures, MAX_BACKOFF_STEPS);
    this.pollTimer = setTimeout(() => {
      this.pollTimer = null;
      if (this.polling) void this.poll();
    }, delay);
  }

  private stopPolling(): void {
    this.polling = false;
    if (this.pollTimer) {
      clearTimeout(this.pollTimer);
      this.pollTimer = null;
    }
  }
}

export const jobs = new JobsStore();
