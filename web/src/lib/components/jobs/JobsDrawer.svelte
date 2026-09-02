<script lang="ts">
  import { jobs } from '$lib/stores/jobs.svelte';
  import { jobDownloadUrl, type Job, type JobItem, type JobItemStatus } from '$lib/api/jobs';
  import Notice from '$lib/components/Notice.svelte';
  import {
    mdiClose,
    mdiChevronDown,
    mdiChevronRight,
    mdiCancel,
    mdiDownload,
    mdiTrashCanOutline
  } from '@mdi/js';
  import { Dialog } from 'bits-ui';
  import { Badge, Button, IconButton, ProgressBar, Text, type Color } from '@immich/ui';

  let expanded = $state<string | null>(null);

  const KIND_LABELS: Record<string, string> = {
    export_immich: 'Export to Immich',
    download_zip: 'Download ZIP',
    apply_preset: 'Apply Preset',
    reset_edits: 'Reset Edits'
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
        return 'text-primary';
      default:
        return 'text-dark/65';
    }
  }

  function itemName(item: JobItem): string {
    return item.result?.filename ?? item.asset_id.slice(0, 8);
  }
</script>

{#if jobs.open}
  <Dialog.Root open onOpenChange={(open) => !open && jobs.close()}>
    <Dialog.Portal>
      <Dialog.Overlay class="fixed inset-0 z-40 bg-black/60 backdrop-blur-[2px]" />
      <Dialog.Content
        class="fixed top-0 right-0 bottom-0 z-50 flex h-dvh max-h-dvh w-105 max-w-[92vw] flex-col border-l border-white/10 bg-neutral-950 pt-[env(safe-area-inset-top)] pb-[env(safe-area-inset-bottom)] shadow-2xl"
      >
        <header
          class="flex h-16 flex-none items-center justify-between border-b border-hairline px-4"
        >
          <div>
            <Dialog.Title class="text-base font-semibold text-white">Jobs</Dialog.Title>
            <p class="mt-0.5 text-[10px] text-white/40">
              {jobs.activeCount > 0 ? `${jobs.activeCount} active` : 'No active jobs'}
            </p>
          </div>
          <Dialog.Close>
            {#snippet child({ props })}
              <IconButton
                {...props}
                size="small"
                variant="ghost"
                color="secondary"
                icon={mdiClose}
                aria-label="Close jobs"
              />
            {/snippet}
          </Dialog.Close>
        </header>

        <div class="flex-1 min-h-0 overflow-y-auto p-4 flex flex-col gap-3">
          {#if jobs.jobs.length === 0}
            <Text size="tiny" color="muted" class="px-1 py-4 text-center">No jobs yet.</Text>
          {/if}

          {#each jobs.jobs as job (job.id)}
            <div class="rounded-md border border-hairline bg-white/4">
              <div class="flex items-center gap-2 px-3 py-3">
                <IconButton
                  size="small"
                  variant="ghost"
                  color="secondary"
                  class="flex-none"
                  icon={expanded === job.id ? mdiChevronDown : mdiChevronRight}
                  onclick={() => toggleExpand(job.id)}
                  aria-label="Toggle details"
                />
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="text-xs font-medium truncate">{kindLabel(job.kind)}</span>
                    <Badge size="tiny" color={statusColor(job.status)}>{job.status}</Badge>
                  </div>
                  <ProgressBar
                    progress={progress(job)}
                    max={100}
                    size="tiny"
                    shape="round"
                    color="primary"
                    class="mt-1"
                    stop={false}
                    aria-label={`${kindLabel(job.kind)} progress`}
                  />
                  <div class="mt-1 text-[10px] text-dark/65">
                    {job.completed + job.failed} / {job.total}
                    {#if job.failed > 0}
                      · <span class="text-red-400">{job.failed} failed</span>
                    {/if}
                  </div>
                </div>
                {#if isActive(job.status)}
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

              {#if expanded === job.id}
                <div class="border-t border-hairline bg-black/15 px-3 py-2.5">
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
          <footer class="flex-none border-t border-hairline p-3">
            <Button
              size="tiny"
              variant="ghost"
              color="secondary"
              fullWidth
              leadingIcon={mdiTrashCanOutline}
              onclick={jobs.clear}
            >
              Clear finished
            </Button>
          </footer>
        {/if}
      </Dialog.Content>
    </Dialog.Portal>
  </Dialog.Root>
{/if}
