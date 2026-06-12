import { cancelJob, getJob, listJobs, type Job, type JobItem } from '$lib/api/jobs';
import { toasts } from '$lib/stores/toasts.svelte';

function isActive(status: Job['status']): boolean {
  return status === 'pending' || status === 'running';
}

class JobsStore {
  jobs = $state<Job[]>([]);
  open = $state(false);
  loading = $state(false);
  items = $state<Record<string, JobItem[]>>({});

  private sources = new Map<string, EventSource>();
  private pollTimer: ReturnType<typeof setInterval> | null = null;

  activeCount = $derived(this.jobs.filter((j) => isActive(j.status)).length);

  load = async (): Promise<void> => {
    if (this.loading) return;
    this.loading = true;
    try {
      this.jobs = await listJobs();
      this.syncStreams();
    } catch (e) {
      toasts.push('error', `Failed to load jobs: ${(e as Error).message}`, 6000);
    } finally {
      this.loading = false;
    }
  };

  toggle = (): void => {
    if (this.open) {
      this.close();
    } else {
      this.open = true;
      void this.load();
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
      toasts.push('error', `Failed to cancel job: ${(e as Error).message}`, 6000);
    }
  };

  loadItems = async (id: string): Promise<void> => {
    try {
      const detail = await getJob(id);
      this.patch(detail.job);
      this.items = { ...this.items, [id]: detail.items };
    } catch (e) {
      toasts.push('error', `Failed to load job items: ${(e as Error).message}`, 6000);
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
    const source = new EventSource(`/api/jobs/${id}/events`);
    source.addEventListener('job', (ev) => {
      try {
        const job = JSON.parse((ev as MessageEvent).data) as Job;
        this.patch(job);
        if (!isActive(job.status)) {
          this.disconnect(job.id);
        }
      } catch {
        /* ignore malformed event */
      }
    });
    source.onerror = () => {
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
    if (this.pollTimer) return;
    this.pollTimer = setInterval(() => {
      if (this.open) void this.load();
    }, 4000);
  }

  private stopPolling(): void {
    if (this.pollTimer) {
      clearInterval(this.pollTimer);
      this.pollTimer = null;
    }
  }
}

export const jobs = new JobsStore();
