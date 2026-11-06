<script lang="ts">
  import { jobsProgress } from '#lib/components/jobs/jobsProgress.svelte.ts';
  import { Button, LoadingSpinner } from '@immich/ui';
</script>

{#if jobsProgress.visible}
  <Button
    variant="ghost"
    color="secondary"
    class="h-auto w-full flex-col items-stretch gap-2 rounded-xl bg-light-100 px-3 py-2.5 text-start text-[13px] font-normal text-light-950"
    title={jobsProgress.text}
    aria-label={jobsProgress.ariaLabel}
    onclick={jobsProgress.toggle}
  >
    <span class="flex min-w-0 items-center gap-2">
      <LoadingSpinner size="tiny" />
      <span class="truncate">{jobsProgress.text}</span>
    </span>
    <span class="block h-1 overflow-hidden rounded-full bg-dark/15" aria-hidden="true">
      <span
        class="block h-full rounded-full bg-primary motion-safe:transition-[width] motion-safe:duration-300"
        style:width="{jobsProgress.fraction * 100}%"
      ></span>
    </span>
  </Button>
{/if}
