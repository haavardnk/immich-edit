<script lang="ts">
  import { Link } from '@immich/ui';
  import type { Snippet } from 'svelte';

  let {
    title,
    more,
    children
  }: {
    title: string;
    more?: { href: string; label: string; name: string };
    children: Snippet;
  } = $props();

  const id = $props.id();
</script>

<section aria-labelledby={id} class="flex flex-col gap-2 py-3">
  <div class="flex min-h-8 items-center justify-between px-4 sm:px-5">
    <h2 {id} class="text-sm font-semibold text-light-950">{title}</h2>
    {#if more}
      <Link
        href={more.href}
        underline={false}
        class="flex min-h-8 items-center text-sm font-medium text-primary touch:min-h-11"
        aria-label={more.name}
      >
        {more.label}
      </Link>
    {/if}
  </div>
  {@render children()}
</section>
