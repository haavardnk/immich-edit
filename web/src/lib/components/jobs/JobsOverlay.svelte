<script lang="ts">
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import JobsPanel from '#lib/components/jobs/JobsPanel.svelte';
  import JobsSheet from '#lib/components/jobs/JobsSheet.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { reveal } from '#lib/motion/reveal.ts';
  import { onEditorRoute } from '#lib/utils/route.ts';
</script>

{#if layout.compact}
  <JobsSheet />
{:else if jobs.open && onEditorRoute()}
  <div
    class="fixed top-0 right-0 z-40 flex h-(--app-height) pt-(--safe-top) pr-(--safe-right) pb-(--safe-bottom) shadow-2xl"
    transition:reveal={{ axis: 'x' }}
  >
    <JobsPanel />
  </div>
{/if}
