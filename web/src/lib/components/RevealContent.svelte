<script lang="ts">
  import { reveal } from '#lib/motion/reveal.ts';
  import { Collapsible } from 'bits-ui';
  import type { Snippet } from 'svelte';

  let { class: className = '', children }: { class?: string; children: Snippet } = $props();
</script>

<Collapsible.Content forceMount>
  {#snippet child({ props, open })}
    {#if open}
      <div {...props} class={className} transition:reveal>
        {@render children()}
      </div>
    {/if}
  {/snippet}
</Collapsible.Content>
