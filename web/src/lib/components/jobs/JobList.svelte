<script lang="ts">
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import JobRow from '#lib/components/jobs/JobRow.svelte';
  import SectionHeading from '#lib/components/SectionHeading.svelte';
  import { jobSections } from '#lib/components/jobs/jobSections.ts';
  import { Text } from '@immich/ui';

  let { grouped = false }: { grouped?: boolean } = $props();

  const sections = $derived(jobSections(jobs.jobs, grouped));
</script>

{#if jobs.jobs.length === 0}
  <Text size="tiny" color="muted" class="px-1 py-4 text-center">No jobs yet.</Text>
{/if}

{#each sections as section (section.label)}
  {#if section.label}
    <SectionHeading title={section.label} level={3} />
  {/if}
  {#each section.jobs as job (job.id)}
    <JobRow {job} />
  {/each}
{/each}
