<script lang="ts">
  import ProgressRing from '#lib/components/jobs/ProgressRing.svelte';
  import { jobsProgressFraction, jobsProgressText } from '#lib/components/jobs/jobFormat.ts';
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiChevronRight } from '@mdi/js';

  const text = $derived(jobsProgressText(jobs.active));
</script>

{#if jobs.active.length > 0}
  <div
    class="pointer-events-none fixed inset-x-0 bottom-[calc(12px+var(--chrome-bottom,var(--safe-bottom)))] z-30 flex justify-center px-4 motion-safe:transition-[bottom] motion-safe:duration-200"
  >
    <Button
      size="small"
      variant="ghost"
      color="secondary"
      shape="round"
      class="pointer-events-auto h-11 max-w-full gap-2.5 bg-light-200/95 ps-2.5 pe-2 text-sm font-normal text-light-950 shadow-popover ring-1 ring-hairline backdrop-blur-md"
      title={text}
      aria-label={`Jobs: ${text}`}
      onclick={jobs.toggle}
    >
      <ProgressRing value={jobsProgressFraction(jobs.active)} />
      <span class="truncate">{text}</span>
      <Icon icon={mdiChevronRight} size="18px" class="shrink-0 text-dark/65" aria-hidden="true" />
    </Button>
  </div>
{/if}
