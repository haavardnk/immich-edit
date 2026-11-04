<script lang="ts">
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import {
    isJobActive,
    jobDownloadUrl,
    type Job,
    type JobItem,
    type JobItemStatus
  } from '#lib/api/jobs.ts';
  import { jobKindLabel, jobProgress } from '#lib/components/jobs/jobFormat.ts';
  import Notice from '#lib/components/Notice.svelte';
  import { mdiChevronDown, mdiChevronRight, mdiCancel, mdiDownload } from '@mdi/js';
  import { Badge, IconButton, ProgressBar, Text, type Color } from '@immich/ui';

  function statusColor(status: Job['status']): Color {
    switch (status) {
      case 'running':
        return 'primary';
      case 'completed':
        return 'success';
      case 'failed':
        return 'danger';
      case 'cancelled':
        return 'secondary';
      default:
        return 'info';
    }
  }

  function itemColor(status: JobItemStatus): string {
    switch (status) {
      case 'completed':
        return 'text-green-400';
      case 'failed':
        return 'text-red-400';
      case 'running':
        return 'text-primary';
      default:
        return 'text-dark/65';
    }
  }

  function itemName(item: JobItem): string {
    return item.result?.filename ?? item.asset_id.slice(0, 8);
  }
</script>

{#if jobs.jobs.length === 0}
  <Text size="tiny" color="muted" class="px-1 py-4 text-center">No jobs yet.</Text>
{/if}

{#each jobs.jobs as job (job.id)}
  <div class="rounded-md border border-hairline bg-light-950/4">
    <div class="flex items-center gap-2 px-3 py-3">
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        class="flex-none"
        icon={jobs.expanded === job.id ? mdiChevronDown : mdiChevronRight}
        onclick={() => jobs.toggleExpanded(job.id)}
        aria-label="Toggle details"
      />
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <span class="text-xs font-medium truncate">{jobKindLabel(job.kind)}</span>
          <Badge size="tiny" color={statusColor(job.status)}>{job.status}</Badge>
        </div>
        <ProgressBar
          progress={jobProgress(job)}
          max={100}
          size="tiny"
          shape="round"
          color="primary"
          class="mt-1"
          stop={false}
          aria-label={`${jobKindLabel(job.kind)} progress`}
        />
        <div class="mt-1 text-[10px] text-dark/65">
          {job.completed + job.failed} / {job.total}
          {#if job.failed > 0}
            · <span class="text-red-400">{job.failed} failed</span>
          {/if}
        </div>
      </div>
      {#if isJobActive(job.status)}
        <IconButton
          size="small"
          variant="ghost"
          color="secondary"
          class="flex-none"
          icon={mdiCancel}
          onclick={() => jobs.cancel(job.id)}
          title="Cancel"
          aria-label="Cancel job"
        />
      {/if}
      {#if job.kind === 'download_zip' && job.status === 'completed' && job.completed > 0}
        <IconButton
          size="small"
          variant="ghost"
          color="secondary"
          class="flex-none"
          icon={mdiDownload}
          href={jobDownloadUrl(job.id)}
          download
          title="Download ZIP"
          aria-label="Download ZIP"
        />
      {/if}
    </div>

    {#if jobs.expanded === job.id}
      <div class="border-t border-hairline bg-recess dark:bg-black/15 px-3 py-2.5">
        {#if jobs.items[job.id] === undefined}
          <p class="text-[11px] text-dark/65">Loading…</p>
        {:else}
          <div class="flex flex-col gap-1">
            {#each jobs.items[job.id] as item (item.id)}
              <div class="min-w-0">
                <div class="flex items-baseline gap-2 text-[11px]">
                  <span class="font-mono truncate flex-1 min-w-0">{itemName(item)}</span>
                  <span class="flex-none {itemColor(item.status)}">{item.status}</span>
                </div>
                {#if item.status === 'failed' && item.error}
                  <Notice message={item.error} class="mt-1 whitespace-pre-wrap" />
                {/if}
                {#each item.result?.warnings ?? [] as warning, i (i)}
                  <Notice color="warning" message={warning} class="mt-1" />
                {/each}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>
{/each}
