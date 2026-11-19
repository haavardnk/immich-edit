import {
  cancelJob,
  clearJobs,
  getJob,
  isJobActive,
  jobDownloadUrl,
  JOB_KINDS,
  jobParamSets,
  jobNeedsAttention,
  listJobs,
  retryJob,
  splitJobs,
  TRANSFER_JOB_KINDS,
  zipReady,
  type Job,
  type JobItem
} from '#lib/api/jobs.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import { downloadUrl } from '#lib/utils/download.ts';
import { countOf } from '#lib/utils/plural.ts';
import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
import { libraryChanged } from '#lib/stores/libraryChange.ts';
import { TOAST_TTL_MS, toasts } from '#lib/stores/toasts.svelte.ts';
import { trash } from '#lib/stores/trash.svelte.ts';
import { endedNow, progressed, revives, supersedes } from '#lib/stores/jobs/transitions.ts';
import { ItemRefresh } from '#lib/stores/jobs/itemRefresh.ts';
import { JobWatch, pollDelay } from '#lib/stores/jobs/watch.ts';
import { jobNotifications } from '#lib/stores/jobs/notify.svelte.ts';

const REFRESHES_THUMBS: ReadonlySet<string> = new Set([
  JOB_KINDS.applyPreset,
  JOB_KINDS.pasteEdits,
  JOB_KINDS.resetEdits
]);

function trashesAssets(job: Job): boolean {
  return (
    job.kind === JOB_KINDS.exportImmich &&
    jobParamSets(job).some(
      (params) => params.replace_original === true || params.on_conflict === 'replace'
    )
  );
}

class JobsStore {
  jobs = $state<Job[]>([]);
  open = $state(false);
  loading = $state(false);
  items = $state<Record<string, JobItem[]>>({});
  watched = $state<string[]>([]);
  attention = $state(false);
  expanded = $state<string | null>(null);

  private downloads = new Set<string>();
  private watch = new JobWatch({
    receive: (job) => this.receive(job),
    isActive: (id) => this.jobs.some((j) => j.id === id && isJobActive(j.status))
  });
  private itemRefresh = new ItemRefresh((id) => this.fetchItems(id));
  private pollTimer: ReturnType<typeof setTimeout> | null = null;
  private polling = false;
  private failures = 0;

  active = $derived(splitJobs(this.jobs).running);
  activeCount = $derived(this.active.length);
  activeTransfers = $derived(this.active.filter((j) => TRANSFER_JOB_KINDS.has(j.kind)).length);
  clearableCount = $derived(this.jobs.length - this.activeCount);

  load = async (): Promise<void> => {
    if (this.loading) return;
    this.loading = true;
    try {
      const fetched = await listJobs();
      const previous = new Map(this.jobs.map((j) => [j.id, j]));
      const merged = fetched.map((job) => {
        const known = previous.get(job.id);
        return supersedes(known, job) ? job : (known ?? job);
      });
      this.jobs = merged;
      this.failures = 0;
      for (const job of merged) {
        const known = previous.get(job.id);
        this.refreshExpanded(known, job);
        if (endedNow(known, job)) this.finish(job);
      }
      this.syncStreams();
    } catch (e) {
      this.failures += 1;
      if (this.failures === 1) toasts.fail('Failed to load jobs', e);
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
    if (isJobActive(job.status)) this.watch.connect(job.id);
    else this.settle(job);
  };

  retry = async (id: string): Promise<void> => {
    let job: Job;
    try {
      job = await retryJob(id);
    } catch (e) {
      toasts.fail('Failed to retry job', e);
      return;
    }
    this.patch(job);
    this.attention = false;
    if (!this.open && !this.watched.includes(id)) this.watched = [id, ...this.watched];
    this.watch.resume(id);
    if (this.items[id] !== undefined) void this.loadItems(id);
  };

  downloadWhenDone = (id: string): void => {
    this.downloads.add(id);
  };

  unwatch = (id: string): void => {
    this.watched = this.watched.filter((w) => w !== id);
  };

  receive = (job: Job): void => {
    const previous = this.jobs.find((j) => j.id === job.id);
    if (!supersedes(previous, job)) return;
    this.patch(job);
    this.refreshExpanded(previous, job);
    if (endedNow(previous, job)) this.finish(job);
    else if (revives(previous, job)) this.watch.connect(job.id);
  };

  private refreshExpanded(previous: Job | undefined, job: Job): void {
    if (this.expanded !== job.id || this.items[job.id] === undefined) return;
    if (progressed(previous, job)) this.itemRefresh.request(job.id);
  }

  private finish(job: Job): void {
    if (REFRESHES_THUMBS.has(job.kind) && job.status === 'completed') {
      void editedThumbs.refresh();
    }
    libraryChanged();
    if (trashesAssets(job)) void this.forgetTrashed(job.id);
    if (this.downloads.delete(job.id) && zipReady(job)) downloadUrl(jobDownloadUrl(job.id));
    this.watch.stop(job.id);
    jobNotifications.notify(job);
    this.settle(job);
  }

  private async forgetTrashed(id: string): Promise<void> {
    let items: JobItem[];
    try {
      items = (await getJob(id)).items;
    } catch {
      return;
    }
    const originals = items
      .filter((item) => item.result?.original_trashed)
      .map((item) => sourceId(item.asset_id));
    const exports = items.flatMap((item) => item.result?.replaced_asset_id ?? []);
    const moved = [
      originals.length > 0 ? countOf(originals.length, 'original') : null,
      exports.length > 0 ? countOf(exports.length, 'earlier export') : null
    ].filter((part) => part !== null);
    if (moved.length === 0) return;
    trash.removeTrashed(
      [...originals, ...exports],
      `Moved ${moved.join(' and ')} to the Immich trash`
    );
  }

  private settle(job: Job): void {
    if (!this.watched.includes(job.id)) return;
    if (jobNeedsAttention(job)) {
      if (!this.open) this.attention = true;
      return;
    }
    setTimeout(() => this.unwatch(job.id), TOAST_TTL_MS.success);
  }

  close = (): void => {
    this.open = false;
    this.stopPolling();
  };

  cancel = async (id: string): Promise<void> => {
    try {
      await cancelJob(id);
    } catch (e) {
      toasts.fail('Failed to cancel job', e);
    }
  };

  clear = async (): Promise<void> => {
    try {
      await clearJobs();
      const removed = new Set(splitJobs(this.jobs).finished.map((j) => j.id));
      for (const id of removed) this.watch.forget(id);
      this.jobs = splitJobs(this.jobs).running;
      this.watched = this.watched.filter((id) => !removed.has(id));
      const items = { ...this.items };
      for (const id of removed) delete items[id];
      this.items = items;
    } catch (e) {
      toasts.fail('Failed to clear jobs', e);
    }
  };

  loadItems = async (id: string): Promise<void> => {
    try {
      await this.fetchItems(id);
    } catch (e) {
      toasts.fail('Failed to load job items', e);
    }
  };

  private async fetchItems(id: string): Promise<void> {
    const detail = await getJob(id);
    this.items = { ...this.items, [id]: detail.items };
    this.receive(detail.job);
  }

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
      if (isJobActive(job.status)) this.watch.connect(job.id);
      else this.watch.stop(job.id);
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
    this.pollTimer = setTimeout(() => {
      this.pollTimer = null;
      if (this.polling) void this.poll();
    }, pollDelay(this.failures));
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
