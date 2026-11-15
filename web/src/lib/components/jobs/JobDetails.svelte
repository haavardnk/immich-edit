<script lang="ts">
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { jobRetryable, type Job, type JobItem } from '#lib/api/jobs.ts';
  import {
    jobItemPreset,
    jobItemState,
    type JobItemState
  } from '#lib/components/jobs/jobFormat.ts';
  import { copyJobErrors } from '#lib/components/jobs/copyErrors.ts';
  import Notice from '#lib/components/Notice.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { mergeProps } from 'bits-ui';
  import { Button } from '@immich/ui';
  import { mdiContentCopy, mdiRefresh } from '@mdi/js';

  let { job, id }: { job: Job; id: string } = $props();

  const items = $derived(jobs.items[job.id]);
  const failures = $derived(items?.some((item) => item.status === 'failed') ?? false);
  const retry = $derived(jobRetryable(job) && layout.mode !== 'touch');

  function itemColor(state: JobItemState): string {
    switch (state) {
      case 'completed':
      case 'replaced':
        return 'text-success-800';
      case 'failed':
        return 'text-danger-800';
      case 'running':
        return 'text-primary';
      case 'skipped':
      case 'duplicate':
        return 'text-warning-800';
      default:
        return 'text-dark/65';
    }
  }

  function itemName(item: JobItem): string {
    return item.result?.filename ?? item.asset_id.slice(0, 8);
  }
</script>

<div {id} class="px-4 pb-3 touch:pb-4">
  {#if items === undefined}
    <p class="text-[11px] text-dark/65">Loading…</p>
  {:else}
    <ul class="flex flex-col gap-1 touch:gap-1.5">
      {#each items as item (item.id)}
        {@const preset = jobItemPreset(job, item)}
        {@const state = jobItemState(item)}
        <li class="min-w-0">
          <div class="flex items-baseline gap-2 text-[11px] touch:text-xs">
            <span class="font-mono truncate flex-1 min-w-0">{itemName(item)}</span>
            {#if preset}
              <TruncationTooltip text={preset}>
                {#snippet child({ props })}
                  <span
                    {...mergeProps(props, { tabindex: -1 })}
                    data-truncate
                    class="max-w-1/3 flex-none truncate text-dark/65"
                  >
                    {preset}
                  </span>
                {/snippet}
              </TruncationTooltip>
            {/if}
            <span class="flex-none {itemColor(state)}">{state}</span>
          </div>
          {#if item.status === 'failed' && item.error}
            <Notice message={item.error} class="mt-1 whitespace-pre-wrap" />
          {/if}
          {#each item.result?.warnings ?? [] as warning, i (i)}
            <Notice color="warning" message={warning} class="mt-1" />
          {/each}
        </li>
      {/each}
    </ul>
  {/if}
  {#if retry || failures}
    <div class="mt-2.5 flex flex-wrap gap-1.5">
      {#if retry}
        <Button
          size="tiny"
          color="primary"
          leadingIcon={mdiRefresh}
          onclick={() => jobs.retry(job.id)}
        >
          Retry {job.failed} failed
        </Button>
      {/if}
      {#if items && failures}
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          class="touch:h-11 touch:text-sm"
          leadingIcon={mdiContentCopy}
          onclick={() => copyJobErrors(job, items)}
        >
          Copy errors
        </Button>
      {/if}
    </div>
  {/if}
</div>
