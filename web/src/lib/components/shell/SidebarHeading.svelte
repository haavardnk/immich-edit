<script lang="ts">
  import type { Snippet } from 'svelte';

  let { class: className = '', children }: { class?: string; children: Snippet } = $props();
</script>

<div class="section-label px-5 pb-2 pt-5 text-dark/65 dark:text-light-950/45 {className}">
  {@render children()}
</div>
