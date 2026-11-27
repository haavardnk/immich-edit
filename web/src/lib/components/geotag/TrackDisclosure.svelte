<script lang="ts">
  import TrackDisclosureHeader from '#lib/components/geotag/TrackDisclosureHeader.svelte';
  import { reveal } from '#lib/motion/reveal.ts';
  import type { Snippet } from 'svelte';

  let {
    label,
    title,
    caption,
    open = $bindable(false),
    children
  }: {
    label: string;
    title: string;
    caption: string;
    open?: boolean;
    children: Snippet;
  } = $props();

  const id = $props.id();
</script>

<section class="flex flex-col border-b border-hairline" aria-label={label}>
  <TrackDisclosureHeader {title} {caption} {open} controls={id} onclick={() => (open = !open)} />
  {#if open}
    <div {id} class="flex flex-col gap-3 px-4 pt-1 pb-3" transition:reveal>
      {@render children()}
    </div>
  {/if}
</section>
