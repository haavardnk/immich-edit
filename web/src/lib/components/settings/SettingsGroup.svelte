<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    title,
    note,
    anchor,
    action,
    children
  }: {
    title: string;
    note?: string;
    anchor?: string;
    action?: Snippet;
    children: Snippet;
  } = $props();

  const id = $props.id();
</script>

<section aria-labelledby="{id}-title" data-setting-anchor={anchor} class="flex flex-col gap-2">
  <div class="flex min-h-6 items-baseline gap-2.5 px-0.5 touch:px-1">
    <h2
      id="{id}-title"
      class="text-label font-semibold text-light-950 touch:section-label touch:text-dark/80"
    >
      {title}
    </h2>
    {#if note}
      <span class="text-xs text-dark/80">{note}</span>
    {/if}
    {#if action}
      <div class="ms-auto flex items-center">{@render action()}</div>
    {/if}
  </div>
  <div
    class="flex flex-col divide-y divide-hairline overflow-hidden rounded-xl bg-light-100 ring-1 ring-hairline touch:rounded-2xl touch:ring-0"
  >
    {@render children()}
  </div>
</section>
