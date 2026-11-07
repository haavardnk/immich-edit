<script lang="ts">
  import ElementTooltip from '#lib/components/ElementTooltip.svelte';
  import type { Snippet } from 'svelte';

  let {
    locked,
    reason,
    class: className = '',
    children
  }: { locked: boolean; reason: string; class?: string; children: Snippet } = $props();
</script>

<ElementTooltip text={reason} disabled={!locked}>
  {#snippet child({ props })}
    <div {...props} class="{className} {locked ? 'cursor-not-allowed *:pointer-events-none' : ''}">
      {@render children()}
    </div>
  {/snippet}
</ElementTooltip>
