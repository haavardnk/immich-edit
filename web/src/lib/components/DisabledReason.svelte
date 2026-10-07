<script lang="ts">
  import { Tooltip } from '@immich/ui';
  import type { Snippet } from 'svelte';

  let {
    locked,
    reason,
    class: className = '',
    children
  }: { locked: boolean; reason: string; class?: string; children: Snippet } = $props();
</script>

<Tooltip text={reason} disabled={!locked}>
  {#snippet child({ props })}
    <div {...props} class="{className} {locked ? 'cursor-not-allowed *:pointer-events-none' : ''}">
      {@render children()}
    </div>
  {/snippet}
</Tooltip>
