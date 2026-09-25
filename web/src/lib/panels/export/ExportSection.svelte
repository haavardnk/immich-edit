<script lang="ts">
  import type { Snippet } from 'svelte';

  let { title, children }: { title: string; children: Snippet } = $props();

  const id = $props.id();
</script>

<section aria-labelledby={id} class="flex flex-col gap-1 border-t border-hairline pt-1.5">
  <h3 {id} class="text-[10px] font-medium tracking-wider text-dark/65 uppercase select-none">
    {title}
  </h3>
  {@render children()}
</section>
