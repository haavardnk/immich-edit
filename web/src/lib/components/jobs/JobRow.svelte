<script lang="ts">
  import { isJobActive, jobNeedsAttention, type Job } from '#lib/api/jobs.ts';
  import JobDetails from '#lib/components/jobs/JobDetails.svelte';
  import { jobDetail, jobKindLabel, jobProgress } from '#lib/components/jobs/jobFormat.ts';
  import { jobKindIcon } from '#lib/components/jobs/jobIcon.ts';
  import JobRowActions from '#lib/components/jobs/JobRowActions.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { Icon, ProgressBar } from '@immich/ui';
  import { mdiChevronDown, mdiChevronRight } from '@mdi/js';

  let { job }: { job: Job } = $props();

  const SEPARATOR = ' · ';

  const label = $derived(jobKindLabel(job.kind));
  const active = $derived(isJobActive(job.status));
  const open = $derived(jobs.expanded === job.id);
  const touch = $derived(layout.mode === 'touch');
  const detailsId = $derived(`job-${job.id}-details`);
  const muted = $derived(job.status === 'cancelled');
  const tile = $derived.by(() => {
    if (!touch) return 'size-5 text-dark/65';
    if (jobNeedsAttention(job)) return 'size-10 rounded-xl bg-danger-500/15 text-danger-800';
    if (job.status === 'completed') return 'size-10 rounded-xl bg-success-500/15 text-success-800';
    return 'size-10 rounded-xl bg-light-200 text-dark/80';
  });
</script>

<div class="border-b border-hairline {open ? 'bg-light-950/4' : ''}">
  <div class="relative">
    <button
      type="button"
      class="absolute inset-0 cursor-pointer hover:bg-light-950/4 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-primary"
      aria-expanded={open}
      aria-controls={open ? detailsId : undefined}
      aria-label="{label} details"
      onclick={() => jobs.toggleExpanded(job.id)}
    ></button>
    <div
      class="pointer-events-none relative flex items-center gap-3 py-2.5 ps-4 pe-2 touch:min-h-16 touch:pe-3"
    >
      <span class="flex flex-none items-center justify-center self-start {tile}">
        <Icon icon={jobKindIcon(job.kind)} size={touch ? '20' : '16'} aria-hidden="true" />
      </span>
      <div class="min-w-0 flex-1">
        <p
          class="truncate text-[13px] font-medium touch:text-[15px] {muted
            ? 'text-dark/65'
            : 'text-light-950'}"
        >
          {label}
        </p>
        {#if active}
          <ProgressBar
            progress={jobProgress(job)}
            max={100}
            size="tiny"
            shape="round"
            color="primary"
            class="my-1.5"
            stop={false}
            aria-label="{label} progress"
          />
        {/if}
        <p class="mt-0.5 text-[11px] text-dark/65 touch:text-[13px]">
          {#each jobDetail(job) as part, i (i)}
            {#if i > 0}{SEPARATOR}{/if}
            <span class={part.danger ? 'text-danger-800' : ''}>{part.text}</span>
          {/each}
        </p>
      </div>
      <div class="pointer-events-auto flex flex-none items-center gap-1.5">
        <JobRowActions {job} />
      </div>
      {#if !touch}
        <Icon
          icon={open ? mdiChevronDown : mdiChevronRight}
          size="16"
          class="flex-none text-dark/65"
          aria-hidden="true"
        />
      {/if}
    </div>
  </div>
  {#if open}
    <JobDetails {job} id={detailsId} />
  {/if}
</div>
