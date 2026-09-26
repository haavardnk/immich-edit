<script lang="ts">
  import { jobDownloadUrl, isJobActive, jobNeedsAttention, type Job } from '$lib/api/jobs';
  import { jobs } from '$lib/stores/jobs.svelte';
  import { jobKindLabel, jobOutcome, jobProgress } from './jobFormat';
  import { Button, IconButton, ProgressBar, ToastContainer, type Color } from '@immich/ui';
  import { mdiClose } from '@mdi/js';

  let { job }: { job: Job } = $props();

  const active = $derived(isJobActive(job.status));
  const failed = $derived(jobNeedsAttention(job));
  const color = $derived<Color>(
    active ? 'primary' : failed ? 'danger' : job.status === 'cancelled' ? 'secondary' : 'success'
  );
  const label = $derived(jobKindLabel(job.kind));
  const zip = $derived(
    job.kind === 'download_zip' && job.status === 'completed' && job.completed > 0
  );
</script>

<div role={failed ? 'alert' : 'status'} aria-live={failed ? 'assertive' : 'polite'}>
  <ToastContainer class="pointer-events-auto px-3" size="small" {color}>
    <div class="flex items-start gap-2">
      <div class="min-w-0 flex-1">
        <p class="truncate text-xs font-medium">{label}</p>
        <p class="mt-0.5 text-[11px] text-dark/70" data-testid="job-toast-outcome">
          {jobOutcome(job)}
        </p>
      </div>
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        icon={mdiClose}
        aria-label={`Dismiss ${label}`}
        onclick={() => jobs.unwatch(job.id)}
      />
    </div>
    {#if active}
      <ProgressBar
        progress={jobProgress(job)}
        max={100}
        size="tiny"
        shape="round"
        color="primary"
        class="mt-2"
        stop={false}
        aria-label={`${label} progress`}
      />
    {:else if failed || zip}
      <div class="mt-2 flex justify-end gap-1">
        {#if zip}
          <Button
            size="tiny"
            variant="ghost"
            color="secondary"
            href={jobDownloadUrl(job.id)}
            download
          >
            Download
          </Button>
        {/if}
        {#if failed}
          <Button size="tiny" variant="ghost" color="danger" onclick={() => jobs.showJob(job.id)}>
            Details
          </Button>
        {/if}
      </div>
    {/if}
  </ToastContainer>
</div>
