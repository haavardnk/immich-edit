<script lang="ts">
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import JobList from '#lib/components/jobs/JobList.svelte';
  import JobListActions from '#lib/components/jobs/JobListActions.svelte';
  import { jobsSummary } from '#lib/components/jobs/jobFormat.ts';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
</script>

{#snippet actions()}
  <JobListActions />
{/snippet}

{#if jobs.open}
  <BottomSheet
    open
    onOpenChange={(open) => !open && jobs.close()}
    title="Jobs"
    subtitle={jobsSummary(jobs.activeCount)}
    close="icon"
    closeLabel="Close jobs"
    detents={[{ fraction: 1 }]}
    bodyClass="flex flex-col pb-3"
    footer={jobs.clearableCount > 0 ? actions : undefined}
  >
    <JobList grouped />
  </BottomSheet>
{/if}
