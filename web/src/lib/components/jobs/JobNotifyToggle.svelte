<script lang="ts">
  import { jobNotifications } from '#lib/stores/jobs/notify.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiBellOffOutline, mdiBellOutline, mdiBellRing } from '@mdi/js';

  const icon = $derived(
    jobNotifications.enabled
      ? mdiBellRing
      : jobNotifications.blocked
        ? mdiBellOffOutline
        : mdiBellOutline
  );
</script>

{#if jobNotifications.supported}
  <IconButton
    size="small"
    variant="ghost"
    color={jobNotifications.enabled ? 'primary' : 'secondary'}
    class="touch:size-11"
    {icon}
    aria-label="Notify when jobs finish"
    title={jobNotifications.blocked
      ? 'Notify when jobs finish: blocked in the browser settings'
      : 'Notify when jobs finish'}
    aria-pressed={jobNotifications.enabled}
    onclick={jobNotifications.toggle}
  />
{/if}
