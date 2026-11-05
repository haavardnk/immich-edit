<script lang="ts">
  import SectionHeading from '#lib/components/SectionHeading.svelte';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
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

{#snippet moreLink()}
  {#if more}
    <TooltipLink
      always
      tooltip={more.name}
      href={more.href}
      aria-label={more.name}
      class="flex min-h-8 items-center rounded-md text-xs font-medium text-primary outline-none hover:underline focus-visible:ring-2 focus-visible:ring-primary touch:min-h-11 touch:text-[12.5px]"
    >
      {more.label}
    </TooltipLink>
  {/if}
{/snippet}

<section aria-labelledby={id} class="flex flex-col gap-1 pb-2">
  <SectionHeading {id} {title} action={more ? moreLink : undefined} />
  {@render children()}
</section>
