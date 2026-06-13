<script lang="ts">
  import { jobs } from '$lib/stores/jobs.svelte';
  import { jobDownloadUrl, type Job, type JobItem, type JobItemStatus } from '$lib/api/jobs';
  import Icon from '$lib/components/Icon.svelte';
  import {
    mdiClose,
    mdiChevronDown,
    mdiChevronRight,
    mdiCancel,
    mdiDownload,
    mdiTrashCanOutline,
  } from '@mdi/js';

  let expanded = $state<string | null>(null);

  const KIND_LABELS: Record<string, string> = {
    export_immich: 'Export to Immich',
    download_zip: 'Download ZIP',
    apply_preset: 'Apply Preset',
    reset_edits: 'Reset Edits',
  };

  function kindLabel(kind: string): string {
    return KIND_LABELS[kind] ?? kind;
  }

  function toggleExpand(id: string): void {
    if (expanded === id) {
      expanded = null;
    } else {
      expanded = id;
      void jobs.loadItems(id);
    }
  }

  function progress(job: Job): number {
    if (job.total === 0) return 0;
    return Math.round(((job.completed + job.failed) / job.total) * 100);
  }

  function statusColor(status: Job['status']): string {
    switch (status) {
      case 'running':
        return 'text-immich-primary';
      case 'completed':
        return 'text-green-400';
      case 'failed':
        return 'text-red-400';
      case 'cancelled':
        return 'text-immich-dark-fg/40';
      default:
        return 'text-immich-dark-fg/60';
    }
  }

  function isActive(status: Job['status']): boolean {
    return status === 'pending' || status === 'running';
  }

  function itemColor(status: JobItemStatus): string {
    switch (status) {
      case 'completed':
        return 'text-green-400';
      case 'failed':
        return 'text-red-400';
      case 'running':
        return 'text-immich-primary';
      default:
        return 'text-immich-dark-fg/40';
    }
  }

  function itemName(item: JobItem): string {
    const result = item.result as { filename?: string } | null;
    return result?.filename ?? item.asset_id.slice(0, 8);
  }
</script>

{#if jobs.open}
  <button
    type="button"
    class="fixed inset-0 z-40 bg-black/40"
    aria-label="Close jobs"
    onclick={jobs.close}
  ></button>
  <aside
    class="fixed right-0 top-0 bottom-0 z-50 w-95 max-w-[90vw] bg-immich-dark-gray border-l border-white/10 shadow-2xl flex flex-col"
  >
    <header class="flex items-center justify-between h-11 px-3 border-b border-white/10 flex-none">
      <span class="text-sm font-semibold">Jobs</span>
      <button
        class="p-1.5 rounded hover:bg-white/10 transition-colors"
        onclick={jobs.close}
        aria-label="Close"
      >
        <Icon path={mdiClose} size={16} />
      </button>
    </header>

    <div class="flex-1 min-h-0 overflow-y-auto p-3 flex flex-col gap-2">
      {#if jobs.jobs.length === 0}
        <p class="text-xs text-immich-dark-fg/50 px-1 py-4 text-center">No jobs yet.</p>
      {/if}

      {#each jobs.jobs as job (job.id)}
        <div class="rounded-lg border border-white/10 bg-black/20">
          <div class="flex items-center gap-2 px-2.5 py-2.5">
            <button
              class="p-0.5 rounded hover:bg-white/10 transition-colors flex-none"
              onclick={() => toggleExpand(job.id)}
              aria-label="Toggle details"
            >
              <Icon path={expanded === job.id ? mdiChevronDown : mdiChevronRight} size={16} />
            </button>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-xs font-medium truncate">{kindLabel(job.kind)}</span>
                <span class="text-[10px] uppercase tracking-wide {statusColor(job.status)}">
                  {job.status}
                </span>
              </div>
              <div class="mt-1 h-1.5 rounded-full bg-white/10 overflow-hidden">
                <div
                  class="h-full bg-immich-primary transition-all"
                  style:width={`${progress(job)}%`}
                ></div>
              </div>
              <div class="mt-1 text-[10px] text-immich-dark-fg/50">
                {job.completed + job.failed} / {job.total}
                {#if job.failed > 0}
                  · <span class="text-red-400">{job.failed} failed</span>
                {/if}
              </div>
            </div>
            {#if isActive(job.status)}
              <button
                class="p-1 rounded hover:bg-white/10 text-immich-dark-fg/60 hover:text-red-400 transition-colors flex-none"
                onclick={() => jobs.cancel(job.id)}
                title="Cancel"
                aria-label="Cancel job"
              >
                <Icon path={mdiCancel} size={16} />
              </button>
            {/if}
            {#if job.kind === 'download_zip' && job.status === 'completed' && job.completed > 0}
              <a
                class="p-1 rounded hover:bg-white/10 text-immich-dark-fg/60 hover:text-immich-primary transition-colors flex-none"
                href={jobDownloadUrl(job.id)}
                download
                title="Download ZIP"
                aria-label="Download ZIP"
              >
                <Icon path={mdiDownload} size={16} />
              </a>
            {/if}
          </div>

          {#if expanded === job.id}
            <div class="border-t border-white/10 px-2.5 py-2">
              {#if jobs.items[job.id] === undefined}
                <p class="text-[11px] text-immich-dark-fg/40">Loading…</p>
              {:else}
                <div class="flex flex-col gap-1">
                  {#each jobs.items[job.id] as item (item.id)}
                    <div class="min-w-0">
                      <div class="flex items-baseline gap-2 text-[11px]">
                        <span class="font-mono truncate flex-1 min-w-0">{itemName(item)}</span>
                        <span class="flex-none {itemColor(item.status)}">{item.status}</span>
                      </div>
                      {#if item.status === 'failed' && item.error}
                        <div
                          class="mt-1 rounded bg-red-500/10 border border-red-500/20 px-2 py-1 text-[10px] leading-snug text-red-300/90 whitespace-pre-wrap wrap-anywhere"
                        >
                          {item.error}
                        </div>
                      {/if}
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>

    {#if jobs.clearableCount > 0}
      <footer class="flex-none border-t border-white/10 p-2">
        <button
          class="w-full flex items-center justify-center gap-2 px-3 py-2 rounded text-xs text-immich-dark-fg/70 hover:bg-white/10 hover:text-red-400 transition-colors"
          onclick={jobs.clear}
        >
          <Icon path={mdiTrashCanOutline} size={16} />
          Clear finished
        </button>
      </footer>
    {/if}
  </aside>
{/if}
