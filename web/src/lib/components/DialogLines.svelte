<script lang="ts">
  import type { DialogTextPart } from '#lib/browse/dialogText.ts';

  let { lines }: { lines: readonly DialogTextPart[][] } = $props();
</script>

{#snippet part(value: DialogTextPart)}
  {#if typeof value === 'string'}{value}{:else}<b class="font-semibold text-light-950">
      {value.strong}
    </b>{/if}
{/snippet}

{#each lines as line, index (index)}
  <p>
    {#each line as value, at (at)}{@render part(value)}{/each}
  </p>
{/each}
