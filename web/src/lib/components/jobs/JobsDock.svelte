<script lang="ts">
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import JobsPanel from '#lib/components/jobs/JobsPanel.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { reveal } from '#lib/motion/reveal.ts';
</script>

{#if jobs.open && !layout.compact}
  <div class="flex h-full flex-none" transition:reveal={{ axis: 'x' }}>
    <JobsPanel />
  </div>
{/if}
