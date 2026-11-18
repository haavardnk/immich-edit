<script lang="ts">
  import { highlightParts, matchRanges } from '#lib/utils/textMatch.ts';

  let { text, query = '' }: { text: string; query?: string } = $props();

  const parts = $derived(highlightParts(text, matchRanges(text, query)));
</script>

{#each parts as part, index (index)}{#if part.match}<mark class="bg-transparent text-primary"
      >{part.text}</mark
    >{:else}{part.text}{/if}{/each}
