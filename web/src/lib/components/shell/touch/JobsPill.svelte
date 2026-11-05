<script lang="ts">
  import { jobsProgressText } from '#lib/components/jobs/jobFormat.ts';
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { Button, LoadingSpinner } from '@immich/ui';
</script>

{#if jobs.active.length > 0}
  <div
    class="pointer-events-none fixed inset-x-0 bottom-[calc(12px+var(--chrome-bottom,var(--safe-bottom)))] z-30 flex justify-center px-4 motion-safe:transition-[bottom] motion-safe:duration-200"
  >
    <Button
      size="small"
      variant="filled"
      color="secondary"
      shape="round"
      class="pointer-events-auto min-h-11 max-w-full gap-2 shadow-popover"
      aria-label={`Jobs: ${jobsProgressText(jobs.active)}`}
      onclick={jobs.toggle}
    >
      <LoadingSpinner size="small" />
      <span class="truncate">{jobsProgressText(jobs.active)}</span>
    </Button>
  </div>
{/if}
