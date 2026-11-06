<script lang="ts">
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { Icon } from '@immich/ui';
  import { mdiContentDuplicate } from '@mdi/js';

  let {
    filename,
    copyBadge,
    class: className = ''
  }: { filename: string; copyBadge: string | null; class?: string } = $props();
</script>

<div class="min-w-0 {className}">
  <TruncationTooltip text={filename}>
    {#snippet child({ props })}
      <h2 {...props} class="truncate text-xs font-semibold touch:text-sm">{filename}</h2>
    {/snippet}
  </TruncationTooltip>
  {#if copyBadge}
    <div
      role="status"
      aria-label="Virtual copy"
      class="inline-flex max-w-full min-w-0 items-center gap-1 align-top font-mono text-[9px] font-medium opacity-70 touch:text-[11px]"
    >
      <Icon icon={mdiContentDuplicate} size="12px" aria-hidden="true" />
      <span class="truncate">{copyBadge}</span>
    </div>
  {/if}
</div>
