import {
  cancelJob,
  clearJobs,
  getJob,
  isJobActive,
  jobNeedsAttention,
  listJobs,
  type Job,
  type JobItem
} from '$lib/api/jobs';
import { url } from '$lib/api/client';
import { editedThumbs } from '$lib/stores/editedThumbs.svelte';
import { toasts } from '$lib/stores/toasts.svelte';

const POLL_BASE_MS = 4000;
const MAX_BACKOFF_STEPS = 4;
const MAX_STREAM_ERRORS = 5;
const DONE_TOAST_MS = 6000;
const REFRESHES_THUMBS = new Set(['apply_preset', 'paste_edits', 'reset_edits']);

class JobsStore {
  jobs = $state<Job[]>([]);
  open = $state(false);
  loading = $state(false);
  items = $state<Record<string, JobItem[]>>({});
  watched = $state<string[]>([]);
  attention = $state(false);
  expanded = $state<string | null>(null);

  private sources = new Map<string, EventSource>();
  private streamErrors = new Map<string, number>();
  private pollTimer: ReturnType<typeof setTimeout> | null = null;
  private polling = false;
  private failures = 0;

  activeCount = $derived(this.jobs.filter((j) => isJobActive(j.status)).length);
  clearableCount = $derived(this.jobs.filter((j) => !isJobActive(j.status)).length);

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
      this.attention = false;
      this.failures = 0;
      this.startPolling();
    }
  };

  showJob = (id: string): void => {
    this.unwatch(id);
    this.expanded = id;
    void this.loadItems(id);
    if (!this.open) this.toggle();
  };

  toggleExpanded = (id: string): void => {
    if (this.expanded === id) {
      this.expanded = null;
      return;
    }
    this.expanded = id;
    void this.loadItems(id);
  };

  track = (job: Job): void => {
    this.patch(job);
    if (!this.watched.includes(job.id)) this.watched = [job.id, ...this.watched];
    if (isJobActive(job.status)) this.connect(job.id);
    else this.settle(job);
  };

  unwatch = (id: string): void => {
    this.watched = this.watched.filter((w) => w !== id);
  };

  receive = (job: Job): void => {
    this.patch(job);
    if (isJobActive(job.status)) return;
    if (REFRESHES_THUMBS.has(job.kind) && job.status === 'completed') {
      void editedThumbs.refresh();
    }
    this.disconnect(job.id);
    this.settle(job);
  };

  private settle(job: Job): void {
    if (!this.watched.includes(job.id)) return;
    if (jobNeedsAttention(job)) {
      if (!this.open) this.attention = true;
      return;
    }
    setTimeout(() => this.unwatch(job.id), DONE_TOAST_MS);
  }

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
      const removed = new Set(this.jobs.filter((j) => !isJobActive(j.status)).map((j) => j.id));
      for (const id of removed) {
        this.disconnect(id);
        this.streamErrors.delete(id);
      }
      this.jobs = this.jobs.filter((j) => isJobActive(j.status));
      this.watched = this.watched.filter((id) => !removed.has(id));
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
      if (isJobActive(job.status)) {
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
      let job: Job;
      try {
        job = JSON.parse((ev as MessageEvent).data) as Job;
      } catch {
        return;
      }
      this.streamErrors.delete(id);
      this.receive(job);
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
