<script lang="ts">
  import type { Snippet } from 'svelte';

  let { class: className = '', children }: { class?: string; children: Snippet } = $props();
</script>

<div
  class="px-5 pb-2 pt-5 text-[10px] font-semibold uppercase text-dark/65 dark:text-light-950/35 {className}"
>
  {@render children()}
</div>
