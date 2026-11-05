<script lang="ts">
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import JobList from '#lib/components/jobs/JobList.svelte';
  import JobListActions from '#lib/components/jobs/JobListActions.svelte';
  import JobsSheet from '#lib/components/jobs/JobsSheet.svelte';
  import { jobsSummary } from '#lib/components/jobs/jobFormat.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { mdiClose } from '@mdi/js';
  import { Dialog } from 'bits-ui';
  import { IconButton } from '@immich/ui';
</script>

{#if layout.compact}
  <JobsSheet />
{:else if jobs.open}
  <Dialog.Root open onOpenChange={(open) => !open && jobs.close()}>
    <Dialog.Portal>
      <Dialog.Overlay class="fixed inset-0 z-40 bg-black/60 backdrop-blur-[2px]" />
      <Dialog.Content
        class="fixed top-0 right-0 bottom-0 z-50 flex h-dvh max-h-dvh w-105 max-w-[92vw] flex-col border-l border-light-950/10 bg-light-50 pt-(--safe-top) pr-(--safe-right) pb-(--safe-bottom) shadow-2xl"
      >
        <header
          class="flex h-16 flex-none items-center justify-between border-b border-hairline px-4"
        >
          <div>
            <Dialog.Title class="text-base font-semibold text-light-950">Jobs</Dialog.Title>
            <p class="mt-0.5 text-[10px] text-light-950/40">
              {jobsSummary(jobs.activeCount)}
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
          <JobList />
        </div>

        {#if jobs.clearableCount > 0}
          <footer class="flex-none border-t border-hairline p-3">
            <JobListActions />
          </footer>
        {/if}
      </Dialog.Content>
    </Dialog.Portal>
  </Dialog.Root>
{/if}
