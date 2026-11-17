<script lang="ts">
  import { Icon } from '@immich/ui';
  import { mdiAlertOutline } from '@mdi/js';

  let { warnings }: { warnings: string[] } = $props();
</script>

{#if warnings.length > 0}
  <ul
    class="space-y-0.5 rounded-sm border border-warning-500/25 bg-warning-500/10 px-2 py-1.5 text-micro leading-snug text-warning-800 touch:text-xs"
  >
    {#each warnings as w (w)}
      <li class="flex items-start gap-1.5">
        <Icon icon={mdiAlertOutline} size="12px" class="mt-0.5 shrink-0" aria-hidden="true" />
        <span>{w}</span>
      </li>
    {/each}
  </ul>
{/if}
