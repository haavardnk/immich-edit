<script lang="ts">
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { tabStatus } from '#lib/components/jobs/tabStatus.ts';
  import { showFaviconBadge } from '#lib/theme/tabIcon.ts';

  const title = document.title;
  const status = $derived(tabStatus(jobs.active, jobs.attention));

  $effect(() => {
    document.title = `${status.prefix}${title}`;
    showFaviconBadge(status.badge);
  });

  $effect(() => () => {
    document.title = title;
    showFaviconBadge(null);
  });
</script>
