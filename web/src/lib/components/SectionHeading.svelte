<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    id,
    title,
    level = 2,
    action
  }: { id?: string; title: string; level?: 2 | 3; action?: Snippet } = $props();
</script>

<div class="flex min-h-8 items-center justify-between gap-2 px-4 pt-2 sm:px-5">
  <svelte:element
    this={`h${level}`}
    {id}
    class="text-[11px] font-medium tracking-[.06em] text-dark/65 uppercase"
  >
    {title}
  </svelte:element>
  {@render action?.()}
</div>
