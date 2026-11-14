<script lang="ts">
  import ProgressRing from '#lib/components/jobs/ProgressRing.svelte';
  import { jobsProgress } from '#lib/components/jobs/jobsProgress.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiChevronRight } from '@mdi/js';
</script>

{#if jobsProgress.visible}
  <div
    class="pointer-events-none fixed inset-x-0 bottom-[calc(12px+var(--chrome-bottom,var(--safe-bottom)))] z-30 flex justify-center px-4 motion-safe:transition-[bottom] motion-safe:duration-200"
  >
    <Button
      size="small"
      variant="ghost"
      color="secondary"
      shape="round"
      class="pointer-events-auto h-11 max-w-full gap-2.5 bg-light-200/95 ps-2.5 pe-2 text-sm font-normal text-light-950 shadow-popover ring-1 ring-hairline backdrop-blur-md"
      title={jobsProgress.text}
      aria-label={jobsProgress.ariaLabel}
      onclick={jobsProgress.toggle}
    >
      <ProgressRing value={jobsProgress.fraction} />
      <span class="truncate">{jobsProgress.text}</span>
      {#if jobsProgress.eta}
        <span class="shrink-0 text-dark/65 tabular-nums">{jobsProgress.eta}</span>
      {/if}
      <Icon icon={mdiChevronRight} size="18px" class="shrink-0 text-dark/65" aria-hidden="true" />
    </Button>
  </div>
{/if}
