<script lang="ts">
  import { isJobActive, jobDownloadUrl, jobRetryable, zipReady, type Job } from '#lib/api/jobs.ts';
  import IconLink from '#lib/components/IconLink.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiClose, mdiDownload, mdiRefresh } from '@mdi/js';

  let { job }: { job: Job } = $props();

  const touch = $derived(layout.mode === 'touch');
  const pill = 'h-9 rounded-full px-3.5 text-sm font-semibold';
  const neutral = `${pill} bg-light-200 text-light-950 hover:bg-light-300`;
  const tinted = `${pill} bg-primary/15 hover:bg-primary/25`;
</script>

{#if isJobActive(job.status)}
  {#if touch}
    <Button
      size="small"
      variant="ghost"
      color="secondary"
      class={neutral}
      aria-label="Cancel job"
      onclick={() => jobs.cancel(job.id)}
    >
      Cancel
    </Button>
  {:else}
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      icon={mdiClose}
      aria-label="Cancel job"
      onclick={() => jobs.cancel(job.id)}
    />
  {/if}
{/if}
{#if zipReady(job)}
  {#if touch}
    <Button
      size="small"
      variant="ghost"
      color="primary"
      class={tinted}
      leadingIcon={mdiDownload}
      href={jobDownloadUrl(job.id)}
      download
    >
      Save ZIP
    </Button>
  {:else}
    <IconLink
      size="small"
      variant="ghost"
      color="secondary"
      icon={mdiDownload}
      href={jobDownloadUrl(job.id)}
      download
      aria-label="Download ZIP"
    />
  {/if}
{/if}
{#if touch && jobRetryable(job)}
  <Button
    size="small"
    variant="ghost"
    color="primary"
    class={tinted}
    leadingIcon={mdiRefresh}
    aria-label="Retry {job.failed} failed"
    onclick={() => jobs.retry(job.id)}
  >
    Retry {job.failed}
  </Button>
{/if}
